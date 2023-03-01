use utoipa::OpenApi;

use super::routes;

#[derive(OpenApi)]
#[openapi(paths(
    routes::bancho_get,
    routes::bancho_post,
    routes::get_screenshot,
    routes::download_beatmapset,
    routes::client_register,
    routes::ask_peppy,
    routes::difficulty_rating,
    routes::osu_error,
    routes::osu_screenshot,
    routes::osu_getfriends,
    routes::osu_getbeatmapinfo,
    routes::osu_getfavourites,
    routes::osu_addfavourite,
    routes::lastfm,
    routes::osu_search,
    routes::osu_search_set,
    routes::osu_submit_modular_selector,
    routes::osu_getreplay,
    routes::osu_rate,
    routes::osu_osz2_getscores,
    routes::osu_comment,
    routes::osu_markasread,
    routes::osu_getseasonal,
    routes::bancho_connect,
    routes::check_updates,
    routes::update_beatmap
))]
pub struct BanchoEndpointsDocs;
