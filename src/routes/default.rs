use crate::database::Database;
use crate::types::{PlayerSessions, TestType};

use actix_web::web::Data;
use actix_web::{get, HttpResponse, Responder};

use std::time::Instant;

/// GET "/"
#[get("/")]
pub async fn index() -> impl Responder {
    let contents = r#"<!DOCTYPE html>
    <html lang="en">
      <head>
        <meta charset="utf-8">
        <title>Hello!</title>
      </head>
      <body>
        <h1>Hello!</h1>
        <p>Hi from Rust</p>
      </body>
    </html>"#;
    HttpResponse::Ok().body(contents)
}

/// GET "/test_pg"
#[get("/test_pg")]
pub async fn test_pg(database: Data<Database>) -> impl Responder {
    let start = Instant::now();
    let contents = database
        .pg
        .get_first_simple(r#"SELECT 'PurePeace' as "name";"#)
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
pub async fn test_async_lock(testdata: Data<TestType>) -> impl Responder {
    let start = Instant::now();
    let mut guard = testdata.write().await;
    *guard += 1;
    // Test io handle (sleep 1s)
    // async_std::task::sleep(std::time::Duration::from_secs(1)).await;
    let end = start.elapsed();
    HttpResponse::Ok()
        .set_header("Content-Type", "text/html; charset=UTF-8")
        .body(&format!("{:?}\n{:.2?}", *guard, end))
}

/// GET "/test_player_read"
#[get("/test_player_read")]
pub async fn test_player_read(player_sessions: Data<PlayerSessions>) -> impl Responder {
    let start = Instant::now();
    let player_sessions = player_sessions.read().await;
    let player_info = match player_sessions.get("test") {
        Some(player) => format!("{:?}", *player),
        None => "non this player".to_string(),
    };
    let end = start.elapsed();
    HttpResponse::Ok().body(format!("{} {:.2?}", player_info, end))
}

/// GET "/test_player_money_add"
#[get("/test_player_money_add")]
pub async fn test_player_money_add(player_sessions: Data<PlayerSessions>) -> impl Responder {
    let start = Instant::now();
    let mut player_sessions = player_sessions.write().await;
    let player_info = match player_sessions.get_mut("test") {
        Some(mut player) => {
            (*player).money += 1;
            async_std::task::sleep(std::time::Duration::from_secs(1)).await;
            format!("{:?}", *player)
        }
        None => "non this player".to_string(),
    };
    let end = start.elapsed();
    HttpResponse::Ok().body(format!("{} {:.2?}", player_info, end))
}

/// GET "/test_player_money_reduce"
#[get("/test_player_money_reduce")]
pub async fn test_player_money_reduce(player_sessions: Data<PlayerSessions>) -> impl Responder {
    let start = Instant::now();
    let mut player_sessions = player_sessions.write().await;
    let player_info = match player_sessions.get_mut("test") {
        Some(mut player) => {
            (*player).money -= 1;
            format!("{:?}", *player)
        }
        None => "non this player".to_string(),
    };
    let end = start.elapsed();
    HttpResponse::Ok().body(format!("{} {:.2?}", player_info, end))
}
