use crate::{database::Database, types::ChannelList};
use crate::{objects::PlayerSessions, utils};

use actix_web::web::{Data, Path, Query};
use actix_web::{get, HttpResponse, Responder};
use async_std::sync::RwLock;
use log::warn;
use maxminddb::{geoip2::City, MaxMindDBError, Reader};
use memmap::Mmap;
use serde_qs;

use std::{
    collections::HashMap,
    net::{Ipv4Addr, Ipv6Addr},
    time::Instant,
};

/// GET "/test_pg"
#[get("/test_pg")]
pub async fn test_pg(database: Data<Database>) -> impl Responder {
    let start = Instant::now();
    let contents = database
        .pg
        .query_first_simple(r#"SELECT 'PurePeace' as "name";"#)
        .await;
    let end = start.elapsed();
    let name: String = contents.get("name");
    HttpResponse::Ok()
        .set_header("Content-Type", "text/html; charset=UTF-8")
        .body(format!("{}\n{:.2?}", name, end))
}

/// GET "/test_redis"
#[get("/test_redis")]
pub async fn test_redis(database: Data<Database>) -> impl Responder {
    let start = Instant::now();
    let _ = database.redis.set("test", &["PurePeace", "NX"]).await;
    let contents: String = database.redis.get("test").await.unwrap();
    let end = start.elapsed();
    HttpResponse::Ok()
        .set_header("Content-Type", "text/html; charset=UTF-8")
        .body(format!("{}\n{:.2?}", contents, end))
}

/// GET "/test_async_lock"
#[get("/test_async_lock")]
pub async fn test_async_lock(player_sessions: Data<RwLock<PlayerSessions>>) -> impl Responder {
    let start = Instant::now();
    let player_sessions = player_sessions.read().await;
    let map = player_sessions.map.read().await;
    let end = start.elapsed();
    HttpResponse::Ok()
        .set_header("Content-Type", "text/html; charset=UTF-8")
        .body(&format!("{:.2?}", end))
}

/// GET "/test_player_read"
#[get("/test_player_read/{token}")]
pub async fn test_player_read(
    token: Path<String>,
    player_sessions: Data<RwLock<PlayerSessions>>,
) -> impl Responder {
    let start = Instant::now();
    let player_info = match player_sessions.read().await.get_player_data(&token.0).await {
        Some(player_data) => format!("{:?}", player_data),
        None => "non this player".to_string(),
    };
    let end = start.elapsed();
    HttpResponse::Ok().body(format!("{} {:.2?}", player_info, end))
}

/// GET "/test_player_money_add"
#[get("/test_player_money_add/{token}")]
pub async fn test_player_money_add(
    token: Path<String>,
    player_sessions: Data<RwLock<PlayerSessions>>,
) -> impl Responder {
    let start = Instant::now();
    let player_sessions = player_sessions.write().await;
    let mut map = player_sessions.map.write().await;
    let player_info = match map.get_mut(&token.0) {
        Some(player) => {
            // (*player).money += 1;
            //async_std::task::sleep(std::time::Duration::from_secs(1)).await;
            format!("{:?}", *player)
        }
        None => "non this player".to_string(),
    };
    let end = start.elapsed();
    HttpResponse::Ok().body(format!("{} {:.2?}", player_info, end))
}

/// GET "/test_player_money_reduce"
#[get("/test_player_money_reduce/{token}")]
pub async fn test_player_money_reduce(
    token: Path<String>,
    player_sessions: Data<RwLock<PlayerSessions>>,
) -> impl Responder {
    let start = Instant::now();
    let player_sessions = player_sessions.write().await;
    let mut map = player_sessions.map.write().await;
    let player_info = match map.get_mut(&token.0) {
        Some(player) => {
            // (*player).money -= 1;
            format!("{:?}", *player)
        }
        None => "non this player".to_string(),
    };
    let end = start.elapsed();
    HttpResponse::Ok().body(format!("{:?} {:.2?}", player_info, end))
}

/// GET "/test_player_money_reduce_special"
#[get("/test_player_money_reduce_special/{token}")]
pub async fn test_player_money_reduce_special(
    token: Path<String>,
    player_sessions: Data<RwLock<PlayerSessions>>,
) -> impl Responder {
    let start = Instant::now();
    let player_info = player_sessions
        .write()
        .await
        .handle_player_get(&token.0, |player| Some(()) /* (*player).money -= 1 */)
        .await;
    let end = start.elapsed();
    HttpResponse::Ok().body(format!("{:?} {:.2?}", player_info, end))
}

/// GET "/pleyer_sessions_all"
#[get("/pleyer_sessions_all")]
pub async fn pleyer_sessions_all(player_sessions: Data<RwLock<PlayerSessions>>) -> impl Responder {
    HttpResponse::Ok().body(player_sessions.read().await.map_to_string().await)
}

/// GET "/pleyer_channels_all"
#[get("/pleyer_channels_all")]
pub async fn player_channels_all(channel_list: Data<RwLock<ChannelList>>) -> impl Responder {
    HttpResponse::Ok().body(format!("{:?}", channel_list.read().await.values()))
}

/// GET "/pleyer_sessions_kick"
#[get("/pleyer_sessions_kick/{token}")]
pub async fn pleyer_sessions_kick(
    token: Path<String>,
    channel_list: Data<RwLock<ChannelList>>,
    player_sessions: Data<RwLock<PlayerSessions>>,
) -> impl Responder {
    HttpResponse::Ok().body(
        match player_sessions
            .write()
            .await
            .logout(&token.0, Some(&channel_list))
            .await
        {
            Some((token, player)) => format!("{}\n{:?}", token, player),
            None => "non this player".to_string(),
        },
    )
}

/// GET "/pleyer_sessions_kick_uid"
#[get("/pleyer_sessions_kick_uid/{user_id}")]
pub async fn pleyer_sessions_kick_uid(
    user_id: Path<i32>,
    channel_list: Data<RwLock<ChannelList>>,
    player_sessions: Data<RwLock<PlayerSessions>>,
) -> impl Responder {
    HttpResponse::Ok().body(
        match player_sessions
            .write()
            .await
            .logout_with_id(user_id.0, Some(&channel_list))
            .await
        {
            Some((token, player)) => format!("{}\n{:?}", token, player),
            None => "non this player".to_string(),
        },
    )
}

/// GET "/test_geo_ip"
#[get("/test_geo_ip/{ip_address}")]
pub async fn test_geo_ip(
    ip_address: Path<String>,
    geo_db: Data<Option<Reader<Mmap>>>,
    query: Query<HashMap<String, String>>,
) -> impl Responder {
    match utils::geo_ip_info(&ip_address.to_string(), &geo_db.get_ref(), &*query).await {
        Ok(json_success) => HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .body(json_success),
        Err(json_error) => HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .body(json_error),
    }
}