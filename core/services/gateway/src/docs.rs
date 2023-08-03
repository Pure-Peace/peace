use super::bancho_endpoints::{BanchoDebugEndpointsDocs, BanchoEndpointsDocs};
use utoipa::OpenApi;

pub struct GatewayApiDocs;

impl GatewayApiDocs {
    pub fn new_docs(debug_endpoints: bool) -> utoipa::openapi::OpenApi {
        let mut docs = BanchoEndpointsDocs::openapi();

        if debug_endpoints {
            docs.merge(BanchoDebugEndpointsDocs::openapi())
        }

        docs
    }
}
