use {
    maxminddb::Reader,
    memmap::Mmap,
    ntex::web::{get, types::Data, types::Path, HttpResponse},
};

#[get("")]
pub async fn index() -> HttpResponse {
    HttpResponse::Ok().body("hello! geoip")
}

#[get("/{ip_address}")]
pub async fn geo_ip(ip_address: Path<String>, geo_db: Data<Option<Reader<Mmap>>>) -> HttpResponse {
    match peace_utils::geoip::geo_ip_info(&ip_address.to_string(), &geo_db.get_ref()).await {
        Ok(json_success) => HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .body(json_success),
        Err(json_error) => HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .body(json_error),
    }
}
