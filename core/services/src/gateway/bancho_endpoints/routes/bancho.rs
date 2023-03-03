use crate::gateway::bancho_endpoints::{
    extractors::{BanchoClientToken, BanchoClientVersion, BanchoRequestBody},
    BanchoHttpError, DynBanchoRoutingService,
};
use axum::{extract::Path, response::Response, routing::*, Extension, Router};
use peace_api::extractors::*;

pub struct BanchoRouter;

impl BanchoRouter {
    pub fn new_router<T: Clone + Sync + Send + 'static>(
        bancho_routing_service: DynBanchoRoutingService,
    ) -> Router<T> {
        Router::new()
            .route("/", get(bancho_get))
            .route("/", post(bancho_post))
            .route("/ss/:screenshot", get(get_screenshot))
            .route("/d/:beatmapset_id", get(download_beatmapset))
            .route("/users", post(client_register))
            .route("/p/doyoureallywanttoaskpeppy", get(ask_peppy))
            .route("/difficulty-rating", get(difficulty_rating))
            .route("/web/osu-error.php", post(osu_error))
            .route("/web/osu-screenshot.php", post(osu_screenshot))
            .route("/web/osu-getfriends.php", get(osu_getfriends))
            .route("/web/osu-getbeatmapinfo.php", get(osu_getbeatmapinfo))
            .route("/web/osu-getfavourites.php", get(osu_getfavourites))
            .route("/web/osu-addfavourite.php", get(osu_addfavourite))
            .route("/web/lastfm.php", get(lastfm))
            .route("/web/osu-search.php", get(osu_search))
            .route("/web/osu-search-set.php", get(osu_search_set))
            .route(
                "/web/osu-submit-modular-selector.php",
                post(osu_submit_modular_selector),
            )
            .route("/web/osu-getreplay.php", get(osu_getreplay))
            .route("/web/osu-rate.php", get(osu_rate))
            .route("/web/osu-osz2-getscores.php", get(osu_osz2_getscores))
            .route("/web/osu-comment.php", post(osu_comment))
            .route("/web/osu-markasread.php", get(osu_markasread))
            .route("/web/osu-getseasonal.php", get(osu_getseasonal))
            .route("/web/bancho_connect.php", get(bancho_connect))
            .route("/web/check-updates", get(check_updates))
            .route("/web/maps/:beatmap_file_name", get(update_beatmap))
            .layer(Extension(bancho_routing_service))
    }
}

/// Bancho get handler
#[utoipa::path(
    get,
    path = "/",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho get handler"),
    )
)]
pub async fn bancho_get(
    Extension(routing_service): Extension<DynBanchoRoutingService>,
) -> Response {
    routing_service.bancho_get().await
}

/// Bancho post handler
#[utoipa::path(
    post,
    path = "/",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho post handler", body = [String]),
    )
)]
pub async fn bancho_post(
    Extension(routing_service): Extension<DynBanchoRoutingService>,
    session_id: Option<BanchoClientToken>,
    version: Option<BanchoClientVersion>,
    ClientIp(ip): ClientIp,
    BanchoRequestBody(body): BanchoRequestBody,
) -> Result<Response, BanchoHttpError> {
    routing_service.bancho_post(session_id, version, ip, body.into()).await
}

/// Bancho get_screenshot
#[utoipa::path(
    get,
    path = "/ss/{screenshot}",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho get_screenshot"),
    )
)]
pub async fn get_screenshot(
    Extension(routing_service): Extension<DynBanchoRoutingService>,
) -> Response {
    routing_service.get_screenshot().await
}

/// Bancho download_beatmapset
#[utoipa::path(
    get,
    path = "/d/{beatmapset_id}",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho download_beatmapset"),
    )
)]
pub async fn download_beatmapset(
    Extension(routing_service): Extension<DynBanchoRoutingService>,
    Path(beatmapset_id): Path<i32>,
) -> Response {
    routing_service.download_beatmapset(beatmapset_id).await
}

/// Bancho client_register
#[utoipa::path(
    post,
    path = "/users",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho client_register", body = [String]),
    )
)]
pub async fn client_register(
    Extension(routing_service): Extension<DynBanchoRoutingService>,
) -> Response {
    routing_service.client_register().await
}

/// Bancho ask_peppy
#[utoipa::path(
    get,
    path = "/p/doyoureallywanttoaskpeppy",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho ask_peppy"),
    )
)]
pub async fn ask_peppy(
    Extension(routing_service): Extension<DynBanchoRoutingService>,
) -> Response {
    routing_service.ask_peppy().await
}

/// Bancho difficulty_rating
#[utoipa::path(
    get,
    path = "/difficulty-rating",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho difficulty_rating"),
    )
)]
pub async fn difficulty_rating(
    Extension(routing_service): Extension<DynBanchoRoutingService>,
) -> Response {
    routing_service.difficulty_rating().await
}

/// Bancho osu_error
#[utoipa::path(
    post,
    path = "/web/osu-error.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho osu_error", body = [String]),
    )
)]
pub async fn osu_error(
    Extension(routing_service): Extension<DynBanchoRoutingService>,
) -> Response {
    routing_service.osu_error().await
}

/// Bancho osu_screenshot
#[utoipa::path(
    post,
    path = "/web/osu-screenshot.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho osu_screenshot", body = [String]),
    )
)]
pub async fn osu_screenshot(
    Extension(routing_service): Extension<DynBanchoRoutingService>,
) -> Response {
    routing_service.osu_screenshot().await
}

/// Bancho osu_getfriends
#[utoipa::path(
    get,
    path = "/web/osu-getfriends.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho osu_getfriends"),
    )
)]
pub async fn osu_getfriends(
    Extension(routing_service): Extension<DynBanchoRoutingService>,
) -> Response {
    routing_service.osu_getfriends().await
}

/// Bancho osu_getbeatmapinfo
#[utoipa::path(
    get,
    path = "/web/osu-getbeatmapinfo.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho osu_getbeatmapinfo"),
    )
)]
pub async fn osu_getbeatmapinfo(
    Extension(routing_service): Extension<DynBanchoRoutingService>,
) -> Response {
    routing_service.osu_getbeatmapinfo().await
}

/// Bancho osu_getfavourites
#[utoipa::path(
    get,
    path = "/web/osu-getfavourites.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho osu_getfavourites"),
    )
)]
pub async fn osu_getfavourites(
    Extension(routing_service): Extension<DynBanchoRoutingService>,
) -> Response {
    routing_service.osu_getfavourites().await
}

/// Bancho osu_addfavourite
#[utoipa::path(
    get,
    path = "/web/osu-addfavourite.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho osu_addfavourite"),
    )
)]
pub async fn osu_addfavourite(
    Extension(routing_service): Extension<DynBanchoRoutingService>,
) -> Response {
    routing_service.osu_addfavourite().await
}

/// Bancho lastfm
#[utoipa::path(
    get,
    path = "/web/osu-lastfm.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho lastfm"),
    )
)]
pub async fn lastfm(
    Extension(routing_service): Extension<DynBanchoRoutingService>,
) -> Response {
    routing_service.lastfm().await
}

/// Bancho osu_search
#[utoipa::path(
    get,
    path = "/web/osu-search.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho osu_search"),
    )
)]
pub async fn osu_search(
    Extension(routing_service): Extension<DynBanchoRoutingService>,
) -> Response {
    routing_service.osu_search().await
}

/// Bancho osu_search_set
#[utoipa::path(
    get,
    path = "/web/osu-search-set.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho osu_search_set"),
    )
)]
pub async fn osu_search_set(
    Extension(routing_service): Extension<DynBanchoRoutingService>,
) -> Response {
    routing_service.osu_search_set().await
}

/// Bancho osu_submit_modular_selector
#[utoipa::path(
    post,
    path = "/web/osu-submit-modular-selector.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho osu_submit_modular_selector", body = [String]),
    )
)]
pub async fn osu_submit_modular_selector(
    Extension(routing_service): Extension<DynBanchoRoutingService>,
) -> Response {
    routing_service.osu_submit_modular_selector().await
}

/// Bancho osu_getreplay
#[utoipa::path(
    get,
    path = "/web/osu-getreplay.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho osu_getreplay"),
    )
)]
pub async fn osu_getreplay(
    Extension(routing_service): Extension<DynBanchoRoutingService>,
) -> Response {
    routing_service.osu_getreplay().await
}

/// Bancho osu_rate
#[utoipa::path(
    get,
    path = "/web/osu-rate.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho osu_rate"),
    )
)]
pub async fn osu_rate(
    Extension(routing_service): Extension<DynBanchoRoutingService>,
) -> Response {
    routing_service.osu_rate().await
}

/// Bancho osu_osz2_getscores
#[utoipa::path(
    get,
    path = "/web/osu-osz2-getscores.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho osu_osz2_getscores"),
    )
)]
pub async fn osu_osz2_getscores(
    Extension(routing_service): Extension<DynBanchoRoutingService>,
) -> Response {
    routing_service.osu_osz2_getscores().await
}

/// Bancho osu_comment
#[utoipa::path(
    post,
    path = "/web/osu-comment.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho osu_comment", body = [String]),
    )
)]
pub async fn osu_comment(
    Extension(routing_service): Extension<DynBanchoRoutingService>,
) -> Response {
    routing_service.osu_comment().await
}

/// Bancho osu_markasread
#[utoipa::path(
    get,
    path = "/web/osu-markasread.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho osu_markasread"),
    )
)]
pub async fn osu_markasread(
    Extension(routing_service): Extension<DynBanchoRoutingService>,
) -> Response {
    routing_service.osu_markasread().await
}

/// Bancho osu_getseasonal
#[utoipa::path(
    get,
    path = "/web/osu-getseasonal.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho osu_getseasonal"),
    )
)]
pub async fn osu_getseasonal(
    Extension(routing_service): Extension<DynBanchoRoutingService>,
) -> Response {
    routing_service.osu_getseasonal().await
}

/// Bancho bancho_connect
#[utoipa::path(
    get,
    path = "/web/bancho_connect.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho bancho_connect"),
    )
)]
pub async fn bancho_connect(
    Extension(routing_service): Extension<DynBanchoRoutingService>,
) -> Response {
    routing_service.bancho_connect().await
}

/// Bancho check_updates
#[utoipa::path(
    get,
    path = "/web/check-updates.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho check_updates"),
    )
)]
pub async fn check_updates(
    Extension(routing_service): Extension<DynBanchoRoutingService>,
) -> Response {
    routing_service.check_updates().await
}

/// Bancho update_beatmap
#[utoipa::path(
    get,
    path = "/web/maps/{beatmap_file_name}",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho update_beatmap"),
    )
)]
pub async fn update_beatmap(
    Extension(routing_service): Extension<DynBanchoRoutingService>,
) -> Response {
    routing_service.update_beatmap().await
}
