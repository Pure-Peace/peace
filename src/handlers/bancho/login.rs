#![allow(unused_variables)]
#![allow(unused_imports)]

use actix_web::web::{Bytes, Data};
use actix_web::{http::HeaderMap, HttpRequest};
use async_std::sync::RwLock;

use crate::objects::{Player, PlayerAddress, PlayerBase, PlayerSessions};
use crate::types::PacketData;
use crate::{constants, database::Database, packets};

use super::parser;
use constants::{packets::LoginReply, Privileges};

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
    let player_base: PlayerBase = match database
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
                    .add(packets::login_reply(LoginReply::UserBanned))
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

    // Lock the PlayerSessions before we handle it
    let player_sessions = player_sessions.write().await;

    // Check is the user_id already login,
    // if true, logout old session
    if player_sessions.user_is_logined(user_id).await {
        // TODO: send notification to old session first
        // Logout old session
        player_sessions.logout_with_id(user_id).await;
        // Send notification to current session
        resp.add(packets::notification(
            "There is another person logging in with your account! ! \n
            Now the server has logged out another session.\n
            If it is not you, please change your password in time.",
        ));
    }

    // Create player object
    let player = Player::from_base(player_base, osu_version, client_info.utc_offset).await;

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

    Ok((
        vec![
            24, 0, 0, 32, 0, 0, 0, 11, 30, 230, 172, 162, 232, 191, 142, 230, 130, 168, 239, 188,
            140, 233, 171, 152, 232, 180, 181, 231, 154, 132, 230, 146, 146, 230, 179, 188, 231,
            137, 185, 105, 0, 0, 7, 0, 0, 0, 11, 5, 80, 101, 97, 99, 101, 24, 0, 0, 44, 0, 0, 0,
            11, 42, 45, 32, 79, 110, 108, 105, 110, 101, 32, 85, 115, 101, 114, 115, 58, 32, 50,
            10, 45, 32, 87, 101, 108, 99, 111, 109, 101, 32, 116, 111, 32, 111, 115, 117, 33, 75,
            97, 102, 117, 117, 126, 126, 92, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 4, 0, 0, 0,
            232, 3, 0, 0, 75, 0, 0, 4, 0, 0, 0, 19, 0, 0, 0, 71, 0, 0, 4, 0, 0, 0, 39, 0, 0, 0, 83,
            0, 0, 30, 0, 0, 0, 232, 3, 0, 0, 11, 9, 80, 117, 114, 101, 80, 101, 97, 99, 101, 32, 0,
            16, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 11, 0, 0, 46, 0, 0, 0, 232, 3, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 202, 7, 224, 54, 0, 0, 0, 0, 100, 112, 123, 63, 41, 0, 0, 0,
            135, 96, 87, 56, 0, 0, 0, 0, 1, 0, 0, 0, 7, 1, 89, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 64, 0,
            0, 6, 0, 0, 0, 11, 4, 35, 111, 115, 117, 64, 0, 0, 11, 0, 0, 0, 11, 9, 35, 97, 110,
            110, 111, 117, 110, 99, 101, 64, 0, 0, 8, 0, 0, 0, 11, 6, 35, 97, 100, 109, 105, 110,
            65, 0, 0, 27, 0, 0, 0, 11, 4, 35, 111, 115, 117, 11, 17, 75, 97, 102, 117, 117, 32,
            103, 108, 111, 98, 97, 108, 32, 99, 104, 97, 116, 2, 0, 65, 0, 0, 31, 0, 0, 0, 11, 9,
            35, 97, 110, 110, 111, 117, 110, 99, 101, 11, 16, 65, 110, 110, 111, 117, 110, 99, 101,
            32, 99, 104, 97, 110, 110, 101, 108, 2, 0, 65, 0, 0, 27, 0, 0, 0, 11, 6, 35, 99, 104,
            105, 110, 97, 11, 15, 67, 104, 105, 110, 97, 32, 99, 111, 109, 109, 117, 110, 105, 116,
            121, 1, 0, 65, 0, 0, 31, 0, 0, 0, 11, 8, 35, 101, 110, 103, 108, 105, 115, 104, 11, 17,
            69, 110, 103, 108, 105, 115, 104, 32, 99, 111, 109, 109, 117, 110, 105, 116, 121, 1, 0,
            65, 0, 0, 26, 0, 0, 0, 11, 6, 35, 97, 100, 109, 105, 110, 11, 14, 65, 114, 101, 32,
            121, 111, 117, 32, 97, 100, 109, 105, 110, 63, 2, 0, 65, 0, 0, 71, 0, 0, 0, 11, 6, 35,
            108, 111, 98, 98, 121, 11, 59, 84, 104, 105, 115, 32, 105, 115, 32, 116, 104, 101, 32,
            108, 111, 98, 98, 121, 32, 119, 104, 101, 114, 101, 32, 121, 111, 117, 32, 102, 105,
            110, 100, 32, 103, 97, 109, 101, 115, 32, 116, 111, 32, 112, 108, 97, 121, 32, 119,
            105, 116, 104, 32, 111, 116, 104, 101, 114, 115, 33, 1, 0, 65, 0, 0, 69, 0, 0, 0, 11,
            7, 35, 114, 97, 110, 107, 101, 100, 11, 56, 82, 97, 110, 107, 32, 114, 101, 113, 117,
            101, 115, 116, 115, 32, 109, 97, 112, 115, 32, 119, 105, 108, 108, 32, 98, 101, 32,
            112, 111, 115, 116, 101, 100, 32, 104, 101, 114, 101, 33, 32, 40, 73, 102, 32, 105,
            116, 115, 32, 114, 97, 110, 107, 101, 100, 46, 41, 1, 0, 72, 0, 0, 6, 0, 0, 0, 1, 0, 0,
            0, 0, 0, 76, 0, 0, 51, 0, 0, 0, 11, 49, 104, 116, 116, 112, 115, 58, 47, 47, 105, 46,
            107, 97, 102, 117, 117, 46, 112, 114, 111, 47, 119, 101, 108, 99, 111, 109, 101, 46,
            112, 110, 103, 124, 104, 116, 116, 112, 115, 58, 47, 47, 107, 97, 102, 117, 117, 46,
            112, 114, 111, 83, 0, 0, 29, 0, 0, 0, 231, 3, 0, 0, 11, 8, 67, 104, 105, 110, 111, 66,
            111, 116, 24, 48, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 83, 0, 0, 30, 0, 0, 0, 232, 3,
            0, 0, 11, 9, 80, 117, 114, 101, 80, 101, 97, 99, 101, 32, 0, 16, 0, 0, 0, 0, 0, 0, 0,
            0, 1, 0, 0, 0, 83, 0, 0, 30, 0, 0, 0, 232, 3, 0, 0, 11, 9, 80, 117, 114, 101, 80, 101,
            97, 99, 101, 32, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0,
        ],
        token,
    ))
}
