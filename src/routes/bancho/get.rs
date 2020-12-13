use super::depends::*;

use crate::renders::BanchoGet;
use askama::Template;

pub async fn handler(counter: Data<IntCounterVec>, render: Data<BanchoGet>) -> HttpResponse {
    counter
        .with_label_values(&["/bancho", "get", "start"])
        .inc();

    HttpResponse::Ok()
        .content_type("text/html")
        .body(render.render().unwrap())
}
