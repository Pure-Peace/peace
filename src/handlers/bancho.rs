use actix_web::{http::HeaderMap, web::Bytes};

pub async fn login(body: &Bytes, request_ip: String, osu_version: String) -> (Bytes, String) {
    (Bytes::from("gg"), "ggg".to_string())
}
