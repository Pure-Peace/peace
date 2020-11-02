use actix_web::web::{Query, Bytes, Data};
use actix_web::{get, HttpResponse, Responder};
use prometheus::IntCounterVec;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Lastfm {
    b: String,
    action: String,
    us: String,
    ha: String
}

#[get("/lastfm.php")]
pub async fn lastfm(Query(query): Query<Lastfm>, counter: Data<IntCounterVec>) -> impl Responder {
    let success = || {
        counter
        .with_label_values(&["/lastfm.php", "get", "success"])
        .inc();
        HttpResponse::Ok().body(Bytes::from("-3"))
    };
    
    // Not flag
    if &query.b[0..1] != "a" {
        return success()
    }



    success()
}
//{'b', 'action', 'us', 'ha'}
