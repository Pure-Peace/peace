use super::bancho_endpoints::BanchoEndpointsDocs;
use utoipa::OpenApi;

pub struct GatewayApiDocs;

impl GatewayApiDocs {
    pub fn new_docs() -> utoipa::openapi::OpenApi {
        let docs = BanchoEndpointsDocs::openapi();
        docs
    }
}
