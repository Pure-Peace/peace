#![allow(unused_variables)]
#![allow(unused_imports)]

use actix_web::web::{Bytes, Data};
use actix_web::{HttpRequest, HttpResponse, Responder};
use async_std::sync::RwLock;
use prometheus::IntCounterVec;

use crate::{
    constants::LoginFailed,
    objects::{Player, PlayerSessions},
    packets,
};
use crate::{database::Database, handlers::bancho};
use crate::{types::ChannelList, utils};

const MAX_FAILED_COUNT: i32 = 4;
const EXPIRE_SECS: i32 = 300;
const DEFAULT_TOKEN: &str = "login_failed";

pub async fn get(req: HttpRequest, body: Bytes, counter: Data<IntCounterVec>) -> impl Responder {
    counter
        .with_label_values(&["/bancho", "get", "start"])
        .inc();
    //println!("GET Body {:?}", &body);
    //println!("REQ {:?}\n--------------", req);
    //let contents = "Hello bancho!";
    HttpResponse::Ok().body(body)
}

pub async fn post(
    req: HttpRequest,
    body: Bytes,
    player_sessions: Data<RwLock<PlayerSessions>>,
    database: Data<Database>,
    channel_list: Data<RwLock<ChannelList>>,
    counter: Data<IntCounterVec>,
) -> impl Responder {
    // Prom counter
    counter
        .with_label_values(&["/bancho", "post", "start"])
        .inc();

    // Get headers
    let headers = req.headers();

    // Get real request ip
    let request_ip = utils::get_realip(&req)
        .await
        .expect("Cannot get request ip");

    // Get osu ver
    let osu_version = utils::get_osuver(&req).await;

    // If not login
    if !headers.contains_key("osu-token") {
        let failed_key = format!("{}-bancho_login_failed", &request_ip);
        let failed_count = database.redis.get(&failed_key).await.unwrap_or(0);
        // Too many failed in 300s, refuse login
        if failed_count > MAX_FAILED_COUNT {
            warn!(
                "ip: {} login refused, beacuse too many login failed_count({}) in {}s;",
                request_ip, failed_count, EXPIRE_SECS
            );
            return HttpResponse::Ok()
                .set_header("cho-token", "login_refused")
                .set_header("cho-protocol", "19")
                .body(
                    packets::PacketBuilder::from(packets::login_reply(LoginFailed::ServerError))
                        .add(packets::notification(
                            "You are not allowed to login now, please wait!",
                        ))
                        .write_out(),
                );
        }

        // Login handle
        let (resp_body, token) = match bancho::login(
            req,
            &body,
            &request_ip,
            osu_version,
            &database,
            &player_sessions,
            &channel_list,
            &counter,
        )
        .await
        {
            Ok((packet_data, token)) => (packet_data, token),
            Err((error_str, packet_builder)) => {
                // Notification string
                let mut failed_notification = String::new();

                // Default failed login reply is InvalidCredentials
                let packet_builder = packet_builder.unwrap_or(packets::PacketBuilder::from(
                    packets::login_reply(LoginFailed::InvalidCredentials),
                ));

                // Record login failed
                counter
                    .with_label_values(&["/bancho", "post", &format!("login.failed.{}", error_str)])
                    .inc();
                // Increase failed count for this ip
                let failed_count: i32 = database.redis.query("INCR", &[&failed_key]).await;
                let _ = database.redis.expire(&failed_key, EXPIRE_SECS).await;

                // Add notification string
                failed_notification.push_str(&format!("Login failed {} time!!\n", failed_count));

                // If reached the retires limit
                if failed_count > MAX_FAILED_COUNT {
                    failed_notification.push_str(&format!("Your login retries have reached the upper limit! \nPlease try again in {} seconds.", EXPIRE_SECS));
                    warn!(
                        "ip: {} login failed, count: {}, will temporarily restrict their login",
                        &request_ip, failed_count
                    );
                } else {
                    failed_notification.push_str(&format!(
                        "You can try {} more times.\n",
                        MAX_FAILED_COUNT - failed_count + 1
                    ))
                };

                // Returns
                (
                    packet_builder
                        .add(packets::notification(&failed_notification))
                        .write_out(),
                    DEFAULT_TOKEN.to_string(),
                )
            }
        };

        return HttpResponse::Ok()
            .set_header("cho-token", token)
            .set_header("cho-protocol", "19")
            .body(resp_body);
    }

    HttpResponse::Ok().body(body)
}
