use crate::bancho_state::DynBanchoStateService;
use axum::{
    response::{IntoResponse, Response},
    routing::*,
    Extension, Router,
};

pub struct BanchoDebugRouter;

impl BanchoDebugRouter {
    pub fn new_router<T: Clone + Sync + Send + 'static>(
        bancho_state_service: DynBanchoStateService,
    ) -> Router<T> {
        Router::new()
            .route("/test", get(test))
            .route("/get_all_sessions", get(get_all_sessions))
            .layer(Extension(bancho_state_service))
    }
}

/// test
#[utoipa::path(
    get,
    path = "/test",
    tag = "bancho_debug",
    responses(
        (status = 200, description = "test"),
    )
)]
pub async fn test() -> Response {
    "ok".into_response()
}

/// get all sessions
#[utoipa::path(
    get,
    path = "/get_all_sessions",
    tag = "bancho_debug",
    responses(
        (status = 200, description = "get all sessions"),
    )
)]
pub async fn get_all_sessions(
    Extension(bancho_state_service): Extension<DynBanchoStateService>,
) -> Response {
    bancho_state_service
        .get_all_sessions()
        .await
        .map(|res| serde_json::to_string_pretty(&res).unwrap().into_response())
        .unwrap_or_else(|err| err.into_response())
}
