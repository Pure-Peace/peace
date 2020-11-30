#![allow(unused_variables)]
#![allow(unused_imports)]

use actix_web::web::{Bytes, Data};
use actix_web::{HttpRequest, HttpResponse, Responder};
use async_std::sync::RwLock;
use prometheus::IntCounterVec;

use crate::utils;
use crate::{
    constants::packets::LoginFailed,
    objects::{Player, PlayerSessions},
    packets,
};
use crate::{database::Database, handlers::bancho};

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
        let failed_count = database.redis.get(failed_key.clone()).await.unwrap_or(0);
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
            player_sessions,
            &counter,
        )
        .await
        {
            Ok((packet_data, token)) => (packet_data, token),
            Err((error_str, packet_data)) => {
                // Record login failed
                counter
                    .with_label_values(&["/bancho", "post", &format!("login.failed.{}", error_str)])
                    .inc();
                // Increase failed count for this ip
                let failed_count: i32 = database.redis.query("INCR", &[failed_key.clone()]).await;
                let _ = database.redis.expire(failed_key, EXPIRE_SECS).await;
                if failed_count > MAX_FAILED_COUNT {
                    warn!(
                        "ip: {} login failed, count: {}, will temporarily restrict their login",
                        &request_ip, failed_count
                    );
                };
                (
                    packet_data.unwrap_or(
                        packets::PacketBuilder::new()
                            .add(packets::login_reply(LoginFailed::InvalidCredentials))
                            .write_out(),
                    ),
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
