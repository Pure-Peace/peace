#![allow(unused_variables)]

use actix_web::web::{Bytes, Data};
use actix_web::HttpRequest;
use async_std::sync::RwLock;
use log::warn;
use regex::Regex;
use std::net::{IpAddr, Ipv4Addr};

use crate::{constants, database::Database, packets};
use crate::{
    objects::{Player, PlayerAddress, PlayerBase, PlayerSessions},
    packets::PacketBuilder,
    utils::argon2_verify,
};
use crate::{
    settings::bancho::BanchoConfig,
    types::{Argon2Cache, ChannelList, PacketData},
};

use super::parser;
use constants::{LoginFailed, Privileges};
use prometheus::IntCounterVec;
use std::time::Instant;

use maxminddb::{geoip2::City, Reader};
use memmap::Mmap;

lazy_static::lazy_static! {
    static ref DEFAULT_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
    static ref OSU_VERSION_REGEX: Regex = Regex::new(r"(?x)(?P<year>\d{4})(?P<month>\d{2})(?P<day>\d{2})").unwrap();
}

#[inline(always)]
/// Bancho login handler
pub async fn login(
    req: HttpRequest,
    body: &Bytes,
    request_ip: &String,
    osu_version: String,
    database: &Data<Database>,
    player_sessions: &Data<RwLock<PlayerSessions>>,
    channel_list: &Data<RwLock<ChannelList>>,
    bancho_config: &BanchoConfig,
    argon2_cache: &Data<RwLock<Argon2Cache>>,
    counter: &Data<IntCounterVec>,
    geo_db: &Data<Option<Reader<Mmap>>>,
) -> Result<(PacketData, String), (&'static str, Option<PacketBuilder>)> {
    let login_start = Instant::now();
    counter
        .with_label_values(&["/bancho", "post", "login.start"])
        .inc();
    // Response packet data
    let mut resp = PacketBuilder::new();

    // Parse login data start ----------
    let parse_start = Instant::now();
    let (username, password_hash, client_info, client_hashes) =
        match parser::parse_login_data(body).await {
            Ok(login_data) => login_data,
            Err(_err_integer) => {
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
    debug!(
        "login data parsed; time spent: {:.2?}; ip: {}, osu_version: {}, username: {};",
        parse_duration, request_ip, osu_version, username
    );

    // Client check
    if bancho_config.client_check && !bancho_config.client_whitelist.contains(&osu_version) {
        if bancho_config.client_only_whitelist {
            warn!(
                "login refused, not allowed osu version: {} username: {}, ip: {}",
                osu_version, username, request_ip
            );
            return Err((
                "not_allowed",
                Some(resp.add(packets::notification("Not allowed osu! version."))),
            ));
        }

        // Version digits
        let version_captures = OSU_VERSION_REGEX.captures(&osu_version);
        if version_captures.is_none() {
            warn!(
                "login refused, invalid osu version: {} username: {}, ip: {}",
                osu_version, username, request_ip
            );
            return Err((
                "not_allowed",
                Some(resp.add(packets::notification("Invalid osu! version."))),
            ));
        }
        let version_captures: i32 = version_captures.unwrap()[0].parse().unwrap();

        // Black list check
        if bancho_config.client_blacklist.contains(&osu_version) {
            warn!(
                "login refused, not allowed osu version: {}; username: {}, ip: {}",
                osu_version, username, request_ip
            );
            return Err((
                "not_allowed",
                Some(resp.add(packets::notification("Not allowed osu! version."))),
            ));
        }

        // Max version check
        if let Some(max_version) = &bancho_config.client_max_version {
            if &version_captures > max_version {
                warn!(
                    "login refused, over than max osu version: {}({} max); username: {}, ip: {}",
                    osu_version, max_version, username, request_ip
                );
                return Err((
                    "not_allowed",
                    Some(resp.add(packets::notification("Not allowed osu! version."))),
                ));
            }
        }

        // Min version check
        if let Some(min_version) = &bancho_config.client_min_version {
            if &version_captures < min_version {
                warn!(
                    "login refused, lower than min osu version: {}({} min); username: {}, ip: {}",
                    osu_version, min_version, username, request_ip
                );
                return Err((
                    "not_allowed",
                    Some(resp.add(packets::notification("osu! version too old."))),
                ));
            }
        }
    }

    // Not allowed username
    if bancho_config.login_disallowed_usernames.contains(&username) {
        warn!(
            "login refused, not allowed username: {}; ip: {}",
            username, request_ip
        );
        return Err(("not_allowed", None));
    }

    // Not allow hashes 1
    let hardware_hashes = client_hashes.adapters_hash.clone() + &client_hashes.disk_id;
    if bancho_config
        .login_disallowed_hardware_hashes
        .contains(&format!("{:x}", md5::compute(&hardware_hashes)))
    {
        warn!(
            "login refused, not allowed hardware hashes: {}; username: {}, ip: {}",
            hardware_hashes, username, request_ip
        );
        return Err(("not_allowed", None));
    }

    // Not allow hashes 2
    if bancho_config
        .login_disallowed_disk_hashes
        .contains(&client_hashes.disk_id)
    {
        warn!(
            "login refused, not allowed disk hash: {}; username: {}, ip: {}",
            client_hashes.disk_id, username, request_ip
        );
        return Err(("not_allowed", None));
    }

    // Not allow hashes 3
    if bancho_config
        .login_disallowed_adapters_hashes
        .contains(&client_hashes.adapters_hash)
    {
        warn!(
            "login refused, not allowed adapters hash: {}; username: {}, ip: {}",
            client_hashes.adapters_hash, username, request_ip
        );
        return Err(("not_allowed", None));
    }

    // Select user base info from database ----------
    let select_base_start = Instant::now();
    // Select from database
    let from_base_row = match database
        .pg
        .query_first(
            r#"SELECT 
                    "id", "name", "privileges", "country", "password"
                    FROM "user"."base" WHERE 
                    "name_safe" = $1;"#,
            &[&username_safe],
        )
        .await
    {
        Ok(from_base_row) => from_base_row,
        Err(_) => {
            warn!(
                "<{}> login failed: account not exists! request_ip: {}; osu_version: {}",
                username, request_ip, osu_version
            );
            return Err(("invalid_credentials", None));
        }
    };
    // Try deserialize player base object
    let mut player_base = match serde_postgres::from_row::<PlayerBase>(&from_base_row) {
        Ok(player_base) => player_base,
        Err(err) => {
            error!(
                "could not deserialize player base: {}; err: {:?}",
                username, err
            );
            return Err(("invalid_credentials", None));
        }
    };
    let select_base_duration = select_base_start.elapsed();
    let user_id = player_base.id;
    let username = player_base.name.clone();
    debug!(
        "success to get player base info {}({}); time spent: {:.2?}; ",
        username, user_id, select_base_duration
    );
    // Not allowed id
    if bancho_config.login_disallowed_id.contains(&user_id) {
        warn!(
            "login refused, not allowed user id: {}; username: {}, ip: {}",
            user_id, username, request_ip
        );
        return Err(("not_allowed", None));
    }

    // Try read password hash from argon2 cache
    let cached_password_hash = {
        argon2_cache
            .read()
            .await
            .get(&player_base.password)
            .cloned()
    };

    // Checking password
    match cached_password_hash {
        // Cache hitted (μs level)
        Some(cached_password_hash) => {
            debug!(
                "password cache hitted: {}({})",
                player_base.name, player_base.id
            );
            if cached_password_hash != password_hash {
                warn!("{} login failed, invalid password (cached)", username);
                return Err(("invalid_credentials", None));
            }
        }
        None => {
            // Cache not hitted, do argon2 verify (ms level)
            let argon2_verify_start = Instant::now();

            let verify_result = argon2_verify(&player_base.password, &password_hash).await;

            let argon2_verify_end = argon2_verify_start.elapsed();
            debug!(
                "Argon2 verify success, time spent: {:.2?};",
                argon2_verify_end
            );

            // Invalid password
            if !verify_result {
                warn!("{} login failed, invalid password (non-cached)", username);
                return Err(("invalid_credentials", None));
            }

            // If password is correct, cache it
            // key = argon2 cipher, key = password hash
            argon2_cache
                .write()
                .await
                .insert(player_base.password.clone(), password_hash);
            debug!(
                "new password cache added: {}({})",
                player_base.name, player_base.id
            );
        }
    };

    // Check user's privileges
    if Privileges::Normal.not_enough(player_base.privileges) {
        warn!(
            "refuse login, because user {}({}) has banned",
            username, user_id
        );
        return Err((
            "user_banned",
            Some(
                resp.add(packets::notification("you have been slained."))
                    .add(packets::login_reply(LoginFailed::UserBanned)),
            ),
        ));
    }

    // Maintenance mode
    if bancho_config.maintenance_enabled && Privileges::Admin.not_enough(player_base.privileges) {
        return Err((
            "maintenance",
            Some(
                resp.add(packets::notification(
                    &bancho_config.maintenance_notification,
                ))
                .add(packets::login_reply(LoginFailed::ServerError)),
            ),
        ));
    }

    // Check user's hardware addresses
    let select_addresses_start = Instant::now();
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
                "could not deserialize player hardware address: {}({}); err: {:?}",
                username, user_id, err
            );
            panic!();
        }),
        Err(err) => {
            error!(
                "user {}({}) login failed, errors when checking hardware addresses; err: {:?}",
                username, user_id, err
            );
            return Err(("checking_hardware_addresses", None));
        }
    };
    let select_addresses_duration = select_addresses_start.elapsed();
    debug!(
        "success to get player addresses info {}({}); time spent: {:.2?}; address count: {}",
        username,
        user_id,
        select_addresses_duration,
        player_addresses.len()
    );

    // PlayerAddress handle
    let (address_id, similarity) = match player_addresses.len() {
        // If not any addresses matched, create it
        0 => {
            let insert_address_start = Instant::now();
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
                        "user {}({}) login failed, errors when insert user address; err: {:?}",
                        username, user_id, err
                    );
                    return Err(("insert user address", None));
                }
            };
            let insert_address_duration = insert_address_start.elapsed();
            info!(
                "success to create a new player address for user {}({}), address id: {}; time spent: {:.2?}",
                username, user_id, address_id, insert_address_duration
            );

            (address_id, 101)
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
                            "Banned account warning - user {}({}) login with an address({}) that was banned user's ({})!",
                            username, user_id, address.id, address.user_id
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
                "user {}({}) login with address id: {}; similarity: {};",
                username, user_id, max_similar_address.id, max_similarity
            );

            // !Multiple account warning
            if max_similar_address.user_id != user_id {
                warn!(
                    "Multi account warning - user {}({}) login with other user({})'s address({});",
                    username, user_id, max_similar_address.user_id, max_similar_address.id
                );
            }

            // TODO: ban muti account?
            if !bancho_config.muti_accounts_allowed {}

            // Create new login record for exists address
            /*  */
            (max_similar_address.id, max_similarity)
        }
    };

    // Verify the user
    if Privileges::Verified.not_enough(player_base.privileges) {
        player_base.privileges |= Privileges::Verified as i32;
        database
            .pg
            .execute(
                r#"UPDATE "user"."base" SET "privileges" = $1 WHERE "id" = $2"#,
                &[&player_base.privileges, &player_base.id],
            )
            .await
            .unwrap_or_else(|err| {
                error!(
                    "failed to update user {}({})'s privileges, error: {:?}",
                    username, user_id, err
                );
                0
            });
        info!(
            "user {}({}) has verified now!",
            player_base.name, player_base.id
        );
    }

    // Create player object
    let mut player = Player::from_base(
        player_base,
        client_info,
        request_ip.clone(),
        address_id,
        similarity,
    )
    .await;

    // Update some player info
    player.update_friends(database).await;

    // Update player's stats
    player.update_stats(database).await;

    // Add login record
    player.create_login_record(database).await;

    // update player's location (ip geo)
    if let Some(reader) = geo_db.get_ref() {
        match reader.lookup::<City>(request_ip.parse().unwrap_or(*DEFAULT_IP)) {
            Ok(geo_data) => match geo_data.location.as_ref().and_then(|location| {
                Some((
                    location.latitude.unwrap_or(0.0),
                    location.longitude.unwrap_or(0.0),
                ))
            }) {
                Some((lati, longi)) => player.location = (lati as f32, longi as f32),
                None => {
                    warn!(
                        "Failed to lookup player {}({})'s ip address location info: {}",
                        player.name, player.id, request_ip
                    );
                }
            },
            Err(_err) => {
                warn!(
                    "Failed to lookup player {}({})'s ip address: {}",
                    player.name, player.id, request_ip
                );
            }
        }
    };

    // User data packet, including player stats and presence
    let user_data_packet = packets::user_data(&player).await;

    // Add response packet data
    resp.add_multiple_ref(&mut [
        packets::login_reply(constants::LoginSuccess::Verified(player.id)),
        packets::protocol_version(19),
        packets::bancho_privileges(player.bancho_privileges),
        user_data_packet.clone(),
        packets::silence_end(0), // TODO: real silence end
        packets::friends_list(&player.friends).await,
    ])
    .await;

    // Notifications
    for n in &bancho_config.login_notifications {
        resp.add_ref(packets::notification(n));
    }

    // Menu icon
    if let Some(menu_icon) = &bancho_config.menu_icon {
        resp.add_ref(packets::main_menu_icon(menu_icon));
    };

    let player_id = player.id;
    let player_priv = player.privileges;

    // Lock the PlayerSessions before we handle it
    let mut player_sessions = player_sessions.write().await;

    // Check is the user_id already login,
    // if true, logout old session
    if player_sessions.user_is_logined(user_id).await {
        // TODO: send notification to old session first
        // Logout old session
        player_sessions
            .logout_with_id(user_id, Some(&channel_list))
            .await;
        // Send notification to current session
        resp.add_ref(packets::notification(
            "There is another person logging in with your account!!\nNow the server has logged out another session.\nIf it is not you, please change your password in time.",
        ));
    }

    // Login player to sessions
    let token = player_sessions.login(player).await;

    // Send new user to online users, and add online users to this new user
    for online_player in player_sessions.token_map.read().await.values() {
        let online_player = online_player.read().await;

        online_player.enqueue(user_data_packet.clone()).await;
        // Add online players to this new player
        resp.add_ref(packets::user_data(&online_player).await);
    }

    // Join player into channel
    for channel in channel_list.read().await.values() {
        // Have not privileges to join the channel
        if (player_priv & channel.read_priv) <= 0 {
            continue;
        }

        // Join player into channel
        if channel.auto_join {
            channel.join(player_id, Some(&*player_sessions)).await;
            resp.add_ref(packets::channel_join(&channel.display_name()));
        }

        // Send channel info to client
        resp.add_ref(channel.channel_info_packet());
    }
    // Release lock
    drop(player_sessions);

    resp.add_ref(packets::channel_info_end());

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
