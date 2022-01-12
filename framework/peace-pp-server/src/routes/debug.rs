use {
    ntex::{
        server::Server,
        web::{get, types::Data, HttpResponse},
    },
    std::time::Instant,
    tokio::sync::mpsc::UnboundedSender,
};

use crate::objects::Caches;

/// GET "/"
#[get("/")]
pub async fn index() -> HttpResponse {
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

/// GET "/server_stop"
#[get("/server_stop")]
pub async fn server_stop(sender: Data<UnboundedSender<Option<Server>>>) -> HttpResponse {
    let start = Instant::now();
    let _ = sender.send(None);
    let end = start.elapsed();
    HttpResponse::Ok().body(format!("server_stop done in: {:?}", end))
}

/// GET "/clear_cache"
#[get("/clear_cache")]
pub async fn clear_cache(caches: Data<Caches>) -> HttpResponse {
    let start = Instant::now();
    caches.pp_beatmap_cache.write().await.clear();
    let end = start.elapsed();
    HttpResponse::Ok().body(format!("clear_cache done in: {:?}", end))
}
