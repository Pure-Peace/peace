use peace_logs::api::AdminAuth;
use peace_logs::api::CommonHandleResponse;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        peace_logs::api::config,
        peace_logs::api::debug_mode,
        peace_logs::api::set_env_filter,
        peace_logs::api::set_level,
        crate::components::responder::shutdown_server
    ),
    components(
        schemas(CommonHandleResponse)
    ),
    modifiers(&AdminAuth)
)]
pub struct PeaceApiAdminEndpointsDocs;
