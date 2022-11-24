use peace_logs::api::AdminAuth;
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

#[derive(OpenApi)]
#[openapi(
    paths(
        peace_logs::api::debug_mode,
        peace_logs::api::set_level
    ),
    components(
        schemas(peace_logs::api::CommonResponse)
    ),
    modifiers(&AdminAuth, &AdminAuth)
)]
pub struct GatewayApiDocs;
