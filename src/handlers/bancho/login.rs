#![allow(unused_variables)]

use actix_web::web::{Bytes, Data};
use actix_web::HttpRequest;
use log::warn;
use std::net::{IpAddr, Ipv4Addr};

use peace_database::Database;

use crate::{
    objects::{Bancho, Caches},
    packets,
};
use crate::{
    objects::{Player, PlayerAddress, PlayerSettings, PlayerStatus},
    packets::PacketBuilder,
    utils,
};
use crate::{settings::bancho::model::BanchoConfigData, types::PacketData};

use super::parser;
use peace_constants::{BanchoPrivileges, LoginFailed, Privileges};
use prometheus::IntCounterVec;
use std::time::Instant;

use maxminddb::Reader;
use memmap::Mmap;

lazy_static::lazy_static! {
    static ref DEFAULT_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
}

#[inline(always)]
/// Bancho login handler
pub async fn login(
    req: HttpRequest,
    body: &Bytes,
    request_ip: &String,
    osu_version: String,
    database: &Data<Database>,
    bancho: &Data<Bancho>,
    global_cache: &Data<Caches>,
    counter: &Data<IntCounterVec>,
    geo_db: &Data<Option<Reader<Mmap>>>,
    cfg: &BanchoConfigData,
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
    debug!(
        "login data parsed; time spent: {:.2?}; ip: {}, osu_version: {}, username: {};",
        parse_duration, request_ip, osu_version, username
    );

    // Client check
    {
        let c = &cfg.client_check;
        if c.enabled && !c.client_whitelist.contains(&osu_version) {
            if c.only_whitelist {
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
            let version_captures =
                peace_constants::regexes::OSU_VERSION_REGEX.captures(&osu_version);
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
            if c.client_blacklist.contains(&osu_version) {
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
            if let Some(max_version) = c.max_version {
                if version_captures > max_version {
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
            if let Some(min_version) = c.min_version {
                if version_captures < min_version {
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
    }

    // Not allowed username
    if cfg.login.disallowed_usernames.contains(&username) {
        warn!(
            "login refused, not allowed username: {}; ip: {}",
            username, request_ip
        );
        return Err(("not_allowed", None));
    }

    // Not allow hashes 1
    let hardware_hashes = client_hashes.adapters_hash.clone() + &client_hashes.disk_id;
    if cfg
        .login
        .disallowed_hardware_hashes
        .contains(&format!("{:x}", md5::compute(&hardware_hashes)))
    {
        warn!(
            "login refused, not allowed hardware hashes: {}; username: {}, ip: {}",
            hardware_hashes, username, request_ip
        );
        return Err(("not_allowed", None));
    }

    // Not allow hashes 2
    if cfg
        .login
        .disallowed_disk_hashes
        .contains(&client_hashes.disk_id)
    {
        warn!(
            "login refused, not allowed disk hash: {}; username: {}, ip: {}",
            client_hashes.disk_id, username, request_ip
        );
        return Err(("not_allowed", None));
    }

    // Not allow hashes 3
    if cfg
        .login
        .disallowed_adapters_hashes
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
    let mut player_base = match utils::get_player_base(&username, &database).await {
        Some(p) => p,
        None => {
            warn!(
                "login failed, could not get playerbase ({}); ip: {}; osu version: {}",
                username, request_ip, osu_version
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
    if cfg.login.disallowed_id.contains(&user_id) {
        warn!(
            "login refused, not allowed user id: {}; username: {}, ip: {}",
            user_id, username, request_ip
        );
        return Err(("not_allowed", None));
    }

    // Checking password
    if !utils::checking_password(&player_base, &password_hash, &global_cache.argon2_cache).await {
        warn!(
            "login refused, failed to checking password; username: {}, ip: {}",
            username, request_ip
        );
        return Err(("invalid_credentials", None));
    }

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
    if cfg.maintenance.enabled && Privileges::Admin.not_enough(player_base.privileges) {
        return Err((
            "maintenance",
            Some(
                resp.add(packets::notification(&cfg.maintenance.notification))
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

    let player_status = match PlayerStatus::from_database(user_id, &database).await {
        Some(player_status) => player_status,
        None => {
            error!(
                "login failed, could not get PlayerStatus ({}); ip: {}; osu version: {}",
                username, request_ip, osu_version
            );
            return Err(("invalid_credentials", None));
        }
    };

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
            if !cfg.mutiaccounts.enabled {}

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

    let player_settings = match PlayerSettings::from_database(user_id, &database).await {
        Some(player_settings) => player_settings,
        None => {
            error!(
                "login failed, could not get PlayerSettings ({}); ip: {}; osu version: {}",
                username, request_ip, osu_version
            );
            return Err(("invalid_credentials", None));
        }
    };

    // Create player object
    let mut player = Player::create(
        player_base,
        player_status,
        player_settings,
        client_info,
        request_ip.clone(),
        address_id,
        similarity,
    )
    .await;

    // all players have in-game "supporter"
    // if config "all_players_have_supporter" is enabled
    if cfg.server.all_players_have_supporter {
        player.bancho_privileges |= BanchoPrivileges::Supporter as i32;
    }

    // Update some player info
    player.update_friends(database).await;

    // Update player's stats
    player.update_stats(database).await;

    // Add login record
    player.create_login_record(database).await;

    // update player's location (ip geo)
    if let Some(geo_db) = geo_db.get_ref() {
        match utils::get_geo_ip_data(request_ip, geo_db) {
            Ok(geo_data) => player.geo_data = geo_data,
            Err(_) => {
                warn!(
                    "Failed to lookup player {}({})'s ip address: {}",
                    player.name, player.id, request_ip
                );
            }
        }
    };

    let using_u_name = player.settings.display_u_name;

    // User data packet, including player stats and presence
    let user_stats_packet = packets::user_stats(&player).await;
    let user_data = PacketBuilder::merge(&mut [
        user_stats_packet.clone(),
        packets::user_presence(&player, false).await,
    ]);
    let user_data_u = PacketBuilder::merge(&mut [
        user_stats_packet,
        packets::user_presence(&player, true).await,
    ]);

    // Add response packet data
    resp.add_multiple_ref(&mut [
        packets::login_reply(peace_constants::LoginSuccess::Verified(player.id)),
        packets::protocol_version(19),
        packets::bancho_privileges(player.bancho_privileges),
        if using_u_name {
            &user_data_u
        } else {
            &user_data
        }
        .clone(),
        packets::silence_end(0), // TODO: real silence end
        packets::friends_list(&player.friends).await,
    ])
    .await;

    // Notifications
    for n in &cfg.login.notifications {
        resp.add_ref(packets::notification(n));
    }

    // Menu icon
    if let Some(menu_icon) = &cfg.menu_icon.get() {
        resp.add_ref(packets::main_menu_icon(menu_icon));
    };

    let player_id = player.id;
    let player_priv = player.privileges;

    // Lock the PlayerSessions before we handle it
    let mut player_sessions = bancho.player_sessions.write().await;

    // Check is the user_id already login,
    // if true, logout old session
    if player_sessions.user_is_logined(user_id).await {
        // TODO: send notification to old session first
        // Logout old session
        player_sessions
            .logout_with_id(user_id, Some(&bancho.channel_list))
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

        online_player
            .enqueue(
                if online_player.settings.display_u_name {
                    &user_data_u
                } else {
                    &user_data
                }
                .clone(),
            )
            .await;
        // Add online players to this new player
        resp.add_ref(packets::user_data(&online_player, using_u_name).await);
    }

    // Join player into channel
    for channel in bancho.channel_list.read().await.values() {
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
