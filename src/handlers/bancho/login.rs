#![allow(unused_variables)]
#![allow(unused_imports)]

use actix_web::web::{Bytes, Data};
use actix_web::{http::HeaderMap, HttpRequest};
use async_std::sync::RwLock;

use crate::objects::{Player, PlayerBase, PlayerSessions};
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
    request_ip: String,
    osu_version: String,
    database: &Data<Database>,
    player_sessions: Data<RwLock<PlayerSessions>>,
    counter: &Data<IntCounterVec>,
) -> (PacketData, String) {
    counter
        .with_label_values(&["/bancho", "post", "login.start"])
        .inc();
    // Response packet data
    let resp = packets::PacketBuilder::new();
    // Default token for login failed
    let default_token = String::from("login_failed");

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
                // Login failed
                return (
                    resp.add(packets::login_reply(
                        constants::packets::LoginReply::InvalidCredentials,
                    ))
                    .write_out(),
                    default_token,
                );
            }
        };
    let parse_duration = parse_start.elapsed();
    // Parse login data end ----------
    let osu_version = client_info.osu_version.clone();
    info!(
        "data parsed; time spent: {:.2?}; ip: {}, osu_version: {}, username: {};",
        parse_duration, request_ip, osu_version, username
    );

    // Select user base info from database
    let player_base = match database
        .pg
        .query_first(
            r#"SELECT 
                "id", "name", "privileges", "country" 
                FROM "user"."base" WHERE 
                "name_safe" = $1 and "password" = $2;"#,
            &[&username.to_lowercase().replace(" ", "_"), &password],
        )
        .await
    {
        Ok(row) => {
            serde_postgres::from_row::<PlayerBase>(&row).expect("could not deserialize player base")
        }
        Err(_) => {
            warn!("{} login failed, invalid credentials", username);
            // Login failed
            return (
                resp.add(packets::login_reply(LoginReply::InvalidCredentials))
                    .write_out(),
                default_token,
            );
        }
    };
    debug!("success to get player base info: {:?}", player_base);
    let user_id = player_base.id;
    let username = player_base.name.clone();

    // Check user's priviliges
    if Privileges::Normal.not_enough(player_base.privileges) {
        warn!(
            "refuse login, beacuse user {}({}) has banned",
            username, user_id
        );
        return (
            resp.add(packets::login_reply(LoginReply::UserBanned))
                .add(packets::notification("you have been slained."))
                .write_out(),
            default_token,
        );
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
    info!("user {}({}) has logged in!", username, user_id);

    /* println!(
        "created a player: {}\nnow sessions:  {:?}",
        token,
        player_sessions.map_to_string().await
    ); */

    /* let packet = packets::PacketBuilder::new()
    .add(packets::notification("hihi"))
    //.add(packets::login_reply(constants::packets::LoginReply::AccountPasswordRest))
    .add(packets::notification("you' re fired"))
    .add(packets::rtx("you' re fired"))
    .add(packets::bancho_restart(3000))
    .done(); */
    //println!("data_lines: {:?}\nclient_info_line: {:?}\nclient_hash_set: {:?}", data_lines, client_info_line, client_hash_set);
    counter
        .with_label_values(&["/bancho", "post", "login.success"])
        .inc();
    (
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
    )
}
