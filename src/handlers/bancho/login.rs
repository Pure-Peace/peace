#![allow(unused_variables)]
#![allow(unused_imports)]

use actix_web::web::{Bytes, Data};
use actix_web::{http::HeaderMap, HttpRequest};
use async_std::sync::RwLock;

use crate::objects::{Player, PlayerAddress, PlayerBase, PlayerSessions};
use crate::types::PacketData;
use crate::{constants, database::Database, packets};

use super::parser;
use constants::{packets::LoginFailed, Privileges};

use prometheus::IntCounterVec;

#[inline(always)]
/// Bancho login handler
pub async fn login(
    req: HttpRequest,
    body: &Bytes,
    request_ip: &String,
    osu_version: String,
    database: &Data<Database>,
    player_sessions: Data<RwLock<PlayerSessions>>,
    counter: &Data<IntCounterVec>,
) -> Result<(PacketData, String), (&'static str, Option<PacketData>)> {
    let login_start = std::time::Instant::now();
    counter
        .with_label_values(&["/bancho", "post", "login.start"])
        .inc();
    // Response packet data
    let resp = packets::PacketBuilder::new();

    // Parse login data start ----------
    let parse_start = std::time::Instant::now();
    let (username, password, client_info, client_hashes) =
        match parser::parse_login_data(body).await {
            Ok(login_data) => login_data,
            Err(err_integer) => {
                error!(
                    "Failed: parse_login_data; request_ip: {}; osu_version: {}",
                    request_ip, osu_version
                );
                return Err(("parse_login_data", None));
            }
        };
    let parse_duration = parse_start.elapsed();
    // Parse login data end ----------
    let osu_version = client_info.osu_version.clone();
    let username_safe = username.to_lowercase().replace(" ", "_");
    info!(
        "data parsed; time spent: {:.2?}; ip: {}, osu_version: {}, username: {};",
        parse_duration, request_ip, osu_version, username
    );

    // Select user base info from database ----------
    let select_base_start = std::time::Instant::now();
    let mut player_base: PlayerBase = match database
        .pg
        .query_first(
            r#"SELECT 
                    "id", "name", "privileges", "country" 
                    FROM "user"."base" WHERE 
                    "name_safe" = $1 and "password" = $2;"#,
            &[&username_safe, &password],
        )
        .await
    {
        Ok(row) => serde_postgres::from_row(&row).unwrap_or_else(|err| {
            error!(
                "could not deserialize player base: {}; err: {:?}",
                username, err
            );
            panic!();
        }),
        Err(_) => {
            warn!("{} login failed, invalid credentials", username);
            return Err(("invalid_credentials", None));
        }
    };
    let select_base_duration = select_base_start.elapsed();
    let user_id = player_base.id;
    let username = player_base.name.clone();
    info!(
        "success to get player base info {}({}); time spent: {:.2?}; ",
        username, user_id, select_base_duration
    );

    // Check user's priviliges
    if Privileges::Normal.not_enough(player_base.privileges) {
        warn!(
            "refuse login, beacuse user {}({}) has banned",
            username, user_id
        );
        return Err((
            "user_banned",
            Some(
                resp.add(packets::notification("you have been slained."))
                    .add(packets::login_reply(LoginFailed::UserBanned))
                    .write_out(),
            ),
        ));
    }

    // Check user's hardware addresses
    let select_addresses_start = std::time::Instant::now();
    let player_addresses: Vec<PlayerAddress> = match database
        .pg
        .query(
            r#"SELECT 
                "user"."address"."id", "user_id", "adapters_hash", "uninstall_id", "disk_id", "privileges" 
                FROM "user"."address" 
                LEFT JOIN "user"."base" 
                    ON "user_id" = "user"."base"."id"
                WHERE 
                    "adapters_hash" = $1 
                    OR "uninstall_id" = $2 
                    OR "disk_id" = $3;"#,
            &[
                &client_hashes.adapters_hash,
                &client_hashes.uninstall_id,
                &client_hashes.disk_id,
            ],
        )
        .await
    {
        Ok(row) => serde_postgres::from_rows(&row).unwrap_or_else(|err| {
            error!(
                "could not deserialize player hardward address: {}; err: {:?}",
                username, err
            );
            panic!();
        }),
        Err(err) => {
            error!(
                "user {} login failed, errors when checking hardware addresses; err: {:?}",
                username, err
            );
            return Err(("checking_hardware_addresses", None));
        }
    };
    let select_addresses_duration = select_addresses_start.elapsed();
    info!(
        "success to get player addresses info {}({}); time spent: {:.2?}; address count: {}",
        username,
        user_id,
        select_addresses_duration,
        player_addresses.len()
    );

    // PlayerAddress handle
    match player_addresses.len() {
        // If not any addresses matched, create it
        0 => {
            let insert_address_start = std::time::Instant::now();
            let address_id: i32 = match database
                .pg
                .query_first(
                    r#"INSERT INTO "user"."address" (
                            "user_id", 
                            "time_offset", 
                            "path", 
                            "adapters", 
                            "adapters_hash", 
                            "uninstall_id", 
                            "disk_id"
                         ) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING "id";"#,
                    &[
                        &user_id,
                        &client_info.utc_offset,
                        &client_hashes.osu_path,
                        &client_hashes.adapters,
                        &client_hashes.adapters_hash,
                        &client_hashes.uninstall_id,
                        &client_hashes.disk_id,
                    ],
                )
                .await
            {
                Ok(row) => row.get("id"),
                Err(err) => {
                    error!(
                        "{} login failed, errors when insert user address; err: {:?}",
                        username, err
                    );
                    return Err(("insert user address", None));
                }
            };
            let insert_address_duration = insert_address_start.elapsed();
            info!(
                "success to create a new player address {}({}), address id: {}; time spent: {:.2?}",
                username, user_id, address_id, insert_address_duration
            );

            // Create new login record for new address
            database
                .pg
                .execute(
                    r#"INSERT INTO "user"."login_records" (
                            "user_id", 
                            "address_id", 
                            "ip", 
                            "version"
                         ) VALUES ($1, $2, $3, $4);"#,
                    &[&user_id, &address_id, &request_ip, &osu_version],
                )
                .await;
        }
        // If any addresses matched
        _ => {
            // Calculate similarity
            let mut similarities: Vec<(i32, &PlayerAddress)> = player_addresses
                .iter()
                .map(|address| {
                    let mut similarity = 0;
                    if address.adapters_hash == client_hashes.adapters_hash {
                        similarity += 30;
                    }
                    if address.uninstall_id == client_hashes.uninstall_id {
                        similarity += 20;
                    }
                    if address.disk_id == client_hashes.disk_id {
                        similarity += 50;
                    }
                    if address.user_id == user_id {
                        similarity += 1;
                    }

                    // !Banned account warning
                    if Privileges::Normal.not_enough(address.privileges) {
                        warn!(
                            "Banned account warning - user({}) login with an address({}) that was banned user's ({})!",
                            user_id, address.id, address.user_id
                        )
                    }

                    (similarity, address)
                })
                .collect();

            // Reverse sort
            similarities.sort_by(|(s1, _), (s2, _)| s2.cmp(&s1));

            // Get the most similar
            let (max_similarity, max_similar_address) = similarities[0];
            info!(
                "user({}) login with address id: {}; similarity: {};",
                user_id, max_similar_address.id, max_similarity
            );

            // !Multiple account warning
            if max_similar_address.user_id != user_id {
                warn!(
                    "Multi account warning - user({}) login with other user({})'s address({});",
                    user_id, max_similar_address.user_id, max_similar_address.id
                );
            }

            // Create new login record for exists address
            database
                .pg
                .execute(
                    r#"INSERT INTO "user"."login_records" (
                    "user_id", 
                    "address_id", 
                    "ip", 
                    "version",
                    "similarity"
                ) VALUES ($1, $2, $3, $4, $5);"#,
                    &[
                        &user_id,
                        &max_similar_address.id,
                        &request_ip,
                        &osu_version,
                        &max_similarity,
                    ],
                )
                .await;
        }
    }

    // Verify the user
    if Privileges::Verified.not_enough(player_base.privileges) {
        player_base.privileges |= Privileges::Verified as i32;
        let _ = database
            .pg
            .execute(
                r#"UPDATE "user"."base" SET "privileges" = $1 WHERE "id" = $2"#,
                &[&player_base.privileges, &player_base.id],
            )
            .await;
        info!(
            "user {}({}) has verified now!",
            player_base.name, player_base.id
        );
    }

    // Create player object
    let player = Player::from_base(player_base, osu_version, client_info.utc_offset).await;

    let resp = resp
        .add(packets::login_reply(
            constants::packets::LoginSuccess::Verified(player.id),
        ))
        .add(packets::protocol_version(19))
        .add(packets::bancho_privileges(player.bancho_privileges))
        .add(packets::notification("Welcome to Peace!"))
        .add(packets::main_menu_icon(
            "https://i.kafuu.pro/welcome.png",
            "https://www.baidu.com",
        ))
        .add(packets::silence_end(0))
        .add(packets::channel_info_end())
        .add(packets::match_join_fail());

    // TODO: add this player presence, stats to all PlayerSessions

    // Lock the PlayerSessions before we handle it
    let player_sessions = player_sessions.write().await;

    // Check is the user_id already login,
    // if true, logout old session
    let resp = if player_sessions.user_is_logined(user_id).await {
        // TODO: send notification to old session first
        // Logout old session
        player_sessions.logout_with_id(user_id).await;
        // Send notification to current session
        resp.add(packets::notification(
            "There is another person logging in with your account!!\nNow the server has logged out another session.\nIf it is not you, please change your password in time.",
        ))
    } else {
        resp
    };

    // Login player to sessions
    let token = player_sessions.login(player).await;

    let login_end = login_start.elapsed();
    info!(
        "user {}({}) has logged in; time spent: {:.2?}",
        username, user_id, login_end
    );

    counter
        .with_label_values(&["/bancho", "post", "login.success"])
        .inc();

    Ok((resp.write_out(), token))
}
