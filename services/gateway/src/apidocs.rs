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
        crate::bancho::handler::bancho_get,
        crate::bancho::handler::bancho_post,
        crate::bancho::handler::get_screenshot,
        crate::bancho::handler::download_beatmapset,
        crate::bancho::handler::client_register,
        crate::bancho::handler::ask_peppy,
        crate::bancho::handler::difficulty_rating,
        crate::bancho::handler::osu_error,
        crate::bancho::handler::osu_screenshot,
        crate::bancho::handler::osu_getfriends,
        crate::bancho::handler::osu_getbeatmapinfo,
        crate::bancho::handler::osu_getfavourites,
        crate::bancho::handler::osu_addfavourite,
        crate::bancho::handler::lastfm,
        crate::bancho::handler::osu_search,
        crate::bancho::handler::osu_search_set,
        crate::bancho::handler::osu_submit_modular_selector,
        crate::bancho::handler::osu_getreplay,
        crate::bancho::handler::osu_rate,
        crate::bancho::handler::osu_osz2_getscores,
        crate::bancho::handler::osu_comment,
        crate::bancho::handler::osu_markasread,
        crate::bancho::handler::osu_getseasonal,
        crate::bancho::handler::bancho_connect,
        crate::bancho::handler::check_updates,
        crate::bancho::handler::update_beatmap
    ),
    components(
        schemas(CommonHandleResponse)
    ),
    modifiers(&AdminAuth)
)]
pub struct GatewayApiDocs;
