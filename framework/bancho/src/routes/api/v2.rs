use ntex::web::{get, types::Data, HttpResponse};
use prometheus::IntCounterVec;

/// GET "/api/v2"
#[get("")]
pub async fn index(counter: Data<IntCounterVec>) -> HttpResponse {
    counter.with_label_values(&["eee", "555", "444"]).inc();
    let contents = "Hello api v2!";
    HttpResponse::Ok().body(contents)
}
