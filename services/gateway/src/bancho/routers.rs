use axum::{
    routing::{get, post},
    Router,
};

use crate::bancho::impls::client;

pub fn bancho_client_routes() -> Router {
    Router::new()
        .route("/", post(client::bancho_post))
        .route("/ss/:screenshot", get(client::get_screenshot))
        .route("/d/:beatmapset_id", get(client::download_beatmapset))
        .route("/users", post(client::client_register))
        .route("/p/doyoureallywanttoaskpeppy", get(client::ask_peppy))
        .route("/difficulty-rating", get(client::difficulty_rating))
        .nest("/web", bancho_client_web_routes())
}

/// osu! bancho `/web` routes.
pub fn bancho_client_web_routes() -> Router {
    Router::new()
        .route("/osu-error.php", post(client::web::osu_error))
        .route("/osu-screenshot.php", post(client::web::osu_screenshot))
        .route("/osu-getfriends.php", get(client::web::osu_getfriends))
        .route("/osu-getbeatmapinfo.php", get(client::web::osu_getbeatmapinfo))
        .route("/osu-getfavourites.php", get(client::web::osu_getfavourites))
        .route("/osu-addfavourite.php", get(client::web::osu_addfavourite))
        .route("/lastfm.php", get(client::web::lastfm))
        .route("/osu-search.php", get(client::web::osu_search))
        .route("/osu-search-set.php", get(client::web::osu_search_set))
        .route(
            "/osu-submit-modular-selector.php",
            post(client::web::osu_submit_modular_selector),
        )
        .route("/osu-getreplay.php", get(client::web::osu_getreplay))
        .route("/osu-rate.php", get(client::web::osu_rate))
        .route("/osu-osz2-getscores.php", get(client::web::osu_osz2_getscores))
        .route("/osu-comment.php", post(client::web::osu_comment))
        .route("/osu-markasread.php", get(client::web::osu_markasread))
        .route("/osu-getseasonal.php", get(client::web::osu_getseasonal))
        .route("/bancho_connect.php", get(client::web::bancho_connect))
        .route("/check-updates", get(client::web::check_updates))
        .route("/maps/:beatmap_file_name", get(client::web::update_beatmap))
}

#[cfg(test)]
mod test {
    use axum::routing::get;
    use axum::Router;
    use axum::{body::Body, extract::Path, http::Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn mock_request() {
        let svr = Router::new().route(
            "/d/:id",
            get(|Path(id): Path<i32>| async move { format!("got id {}", id) }),
        );
        let res = svr
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/d/1123")
                    .header("X-Custom-Foo", "Bar")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await;
        let body =
            hyper::body::to_bytes(res.unwrap().into_body()).await.unwrap();
        println!("res body: {:?}", body);
    }
}
