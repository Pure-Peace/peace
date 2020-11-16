use crate::constants::types::TestType;
use crate::database::Database;

use actix_web::web::Data;
use actix_web::{get, HttpResponse, Responder};

use std::time::Instant;

/* lazy_static! {
  static ref TESTS: Vec<> = 42
} */

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
        .get_all_simple(r#"SELECT name FROM students WHERE id = 1;"#)
        .await;
    let end = start.elapsed();
    let mut name: String = contents[0].get("name");
    name.push_str("\n");
    name.push_str(&format!("{:.2?}", end));
    HttpResponse::Ok()
        .set_header("Content-Type", "text/html; charset=UTF-8")
        .body(name)
}

/// GET "/test_redis"
#[get("/test_redis")]
pub async fn test_redis(database: Data<Database>) -> impl Responder {
    let start = Instant::now();
    let mut contents: String = database.redis.get("test").await.unwrap();
    let end = start.elapsed();
    contents.push_str("\n");
    contents.push_str(&format!("{:.2?}", end));
    HttpResponse::Ok()
        .set_header("Content-Type", "text/html; charset=UTF-8")
        .body(contents)
}

/// GET "/test_async_lock"
#[get("/test_async_lock")]
pub async fn test_async_lock(testdata: Data<TestType>) -> impl Responder {
    let start = Instant::now();
    let mut guard = testdata.lock().await;
    *guard += 1;
    let end = start.elapsed();
    HttpResponse::Ok()
        .set_header("Content-Type", "text/html; charset=UTF-8")
        .body(&format!("{:?} {:.2?}", *guard, end))
}
