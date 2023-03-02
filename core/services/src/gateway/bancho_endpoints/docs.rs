use utoipa::OpenApi;

use super::routes::{bancho, debug};

#[derive(OpenApi)]
#[openapi(paths(
    bancho::bancho_get,
    bancho::bancho_post,
    bancho::get_screenshot,
    bancho::download_beatmapset,
    bancho::client_register,
    bancho::ask_peppy,
    bancho::difficulty_rating,
    bancho::osu_error,
    bancho::osu_screenshot,
    bancho::osu_getfriends,
    bancho::osu_getbeatmapinfo,
    bancho::osu_getfavourites,
    bancho::osu_addfavourite,
    bancho::lastfm,
    bancho::osu_search,
    bancho::osu_search_set,
    bancho::osu_submit_modular_selector,
    bancho::osu_getreplay,
    bancho::osu_rate,
    bancho::osu_osz2_getscores,
    bancho::osu_comment,
    bancho::osu_markasread,
    bancho::osu_getseasonal,
    bancho::bancho_connect,
    bancho::check_updates,
    bancho::update_beatmap
))]
pub struct BanchoEndpointsDocs;

#[derive(OpenApi)]
#[openapi(paths(debug::test, debug::get_all_sessions,))]
pub struct BanchoDebugEndpointsDocs;
