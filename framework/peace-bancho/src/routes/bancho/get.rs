use super::depends::*;

pub async fn handler(counter: Data<IntCounterVec>, bancho: Data<Bancho>) -> HttpResponse {
    counter
        .with_label_values(&["/bancho", "get", "start"])
        .inc();

    let mut render = write_lock!(bancho.render_get);
    render.update().await;

    HttpResponse::Ok()
        .content_type("text/html")
        .body(render.render().unwrap())
}
