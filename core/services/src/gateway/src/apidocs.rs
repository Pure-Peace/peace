use peace_logs::api::AdminAuth;
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use peace_logs::api::CommonHandleResponse;

#[derive(OpenApi)]
#[openapi(
    paths(
        peace_logs::api::config,
        peace_logs::api::debug_mode,
        peace_logs::api::set_env_filter,
        peace_logs::api::set_level,
        peace_api::responder::shutdown_server,
        crate::bancho::routes::bancho_get,
        crate::bancho::routes::bancho_post,
        crate::bancho::routes::get_screenshot,
        crate::bancho::routes::download_beatmapset,
        crate::bancho::routes::client_register,
        crate::bancho::routes::ask_peppy,
        crate::bancho::routes::difficulty_rating,
        crate::bancho::routes::osu_error,
        crate::bancho::routes::osu_screenshot,
        crate::bancho::routes::osu_getfriends,
        crate::bancho::routes::osu_getbeatmapinfo,
        crate::bancho::routes::osu_getfavourites,
        crate::bancho::routes::osu_addfavourite,
        crate::bancho::routes::lastfm,
        crate::bancho::routes::osu_search,
        crate::bancho::routes::osu_search_set,
        crate::bancho::routes::osu_submit_modular_selector,
        crate::bancho::routes::osu_getreplay,
        crate::bancho::routes::osu_rate,
        crate::bancho::routes::osu_osz2_getscores,
        crate::bancho::routes::osu_comment,
        crate::bancho::routes::osu_markasread,
        crate::bancho::routes::osu_getseasonal,
        crate::bancho::routes::bancho_connect,
        crate::bancho::routes::check_updates,
        crate::bancho::routes::update_beatmap
    ),
    components(
        schemas(CommonHandleResponse)
    ),
    modifiers(&AdminAuth)
)]
pub struct GatewayApiDocs;
