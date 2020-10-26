use actix_web::{get, HttpResponse, Responder};

/// GET "/api/v1"
#[get("")]
pub async fn index() -> impl Responder {
    let contents = "Hello api v1!";
    HttpResponse::Ok().body(contents)
}

/// GET "/api/v1/is_online"
#[get("/is_online")]
pub async fn is_online() -> impl Responder {
    let contents = "Hello api v1!";
    HttpResponse::Ok().body(contents)
}

/// GET "/api/v1/online_users"
#[get("/online_users")]
pub async fn online_users() -> impl Responder {
    let contents = "Hello api v1!";
    HttpResponse::Ok().body(contents)
}

/// GET "/api/v1/server_status"
#[get("/server_status")]
pub async fn server_status() -> impl Responder {
    let contents = "Hello api v1!";
    HttpResponse::Ok().body(contents)
}

/// GET "/api/v1/verified_status"
#[get("/verified_status")]
pub async fn verified_status() -> impl Responder {
    let contents = "Hello api v1!";
    HttpResponse::Ok().body(contents)
}

/// GET "/api/v1/ci_trigger"
#[get("/ci_trigger")]
pub async fn ci_trigger() -> impl Responder {
    let contents = "Hello api v1!";
    HttpResponse::Ok().body(contents)
}

/// GET "/api/v1/bot_message"
#[get("/bot_message")]
pub async fn bot_message() -> impl Responder {
    let contents = "Hello api v1!";
    HttpResponse::Ok().body(contents)
}
