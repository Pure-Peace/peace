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
        peace_logs::api::set_level
    ),
    components(
        schemas(CommonHandleResponse)
    ),
    modifiers(&AdminAuth, &AdminAuth)
)]
pub struct GatewayApiDocs;
