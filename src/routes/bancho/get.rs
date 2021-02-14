use super::depends::*;

use crate::renders::BanchoGet;
use askama::Template;

pub async fn handler(counter: Data<IntCounterVec>, render: Data<RwLock<BanchoGet>>) -> HttpResponse {
    counter
        .with_label_values(&["/bancho", "get", "start"])
        .inc();

    let mut render = render.write().await;
    render.update().await;
    
    HttpResponse::Ok()
        .content_type("text/html")
        .body(render.render().unwrap())
}
