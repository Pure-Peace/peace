use peace_logs::api::{AdminAuth, CommonHandleResponse};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

#[derive(OpenApi)]
#[openapi(
    paths(
        peace_logs::api::config,
        peace_logs::api::debug_mode,
        peace_logs::api::set_env_filter,
        peace_logs::api::set_level,
        peace_api::responder::shutdown_server,
        super::bancho_endpoints::routes::bancho_get,
        super::bancho_endpoints::routes::bancho_post,
        super::bancho_endpoints::routes::get_screenshot,
        super::bancho_endpoints::routes::download_beatmapset,
        super::bancho_endpoints::routes::client_register,
        super::bancho_endpoints::routes::ask_peppy,
        super::bancho_endpoints::routes::difficulty_rating,
        super::bancho_endpoints::routes::osu_error,
        super::bancho_endpoints::routes::osu_screenshot,
        super::bancho_endpoints::routes::osu_getfriends,
        super::bancho_endpoints::routes::osu_getbeatmapinfo,
        super::bancho_endpoints::routes::osu_getfavourites,
        super::bancho_endpoints::routes::osu_addfavourite,
        super::bancho_endpoints::routes::lastfm,
        super::bancho_endpoints::routes::osu_search,
        super::bancho_endpoints::routes::osu_search_set,
        super::bancho_endpoints::routes::osu_submit_modular_selector,
        super::bancho_endpoints::routes::osu_getreplay,
        super::bancho_endpoints::routes::osu_rate,
        super::bancho_endpoints::routes::osu_osz2_getscores,
        super::bancho_endpoints::routes::osu_comment,
        super::bancho_endpoints::routes::osu_markasread,
        super::bancho_endpoints::routes::osu_getseasonal,
        super::bancho_endpoints::routes::bancho_connect,
        super::bancho_endpoints::routes::check_updates,
        super::bancho_endpoints::routes::update_beatmap
    ),
    components(
        schemas(CommonHandleResponse)
    ),
    modifiers(&AdminAuth)
)]
pub struct GatewayApiDocs;
