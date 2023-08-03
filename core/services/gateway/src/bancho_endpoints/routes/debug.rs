use axum::{
    response::{IntoResponse, Response},
    routing::*,
    Extension, Router,
};
use core_bancho_state::DynBanchoStateService;
use peace_pb::bancho_state::UserData;
use serde_json::{Map, Value};

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
    #[derive(Serialize)]
    struct AllSessions {
        len: u64,
        indexed_by_session_id: Vec<Map<String, Value>>,
        indexed_by_user_id: Vec<Map<String, Value>>,
        indexed_by_username: Vec<Map<String, Value>>,
        indexed_by_username_unicode: Vec<Map<String, Value>>,
    }

    fn convert(input: Vec<UserData>) -> Vec<Map<String, Value>> {
        input
            .into_iter()
            .map(|i| serde_json::from_str(&i.json).unwrap())
            .collect()
    }

    bancho_state_service
        .get_all_sessions()
        .await
        .map(|res| {
            serde_json::to_string_pretty(&AllSessions {
                len: res.len,
                indexed_by_session_id: convert(res.indexed_by_session_id),
                indexed_by_user_id: convert(res.indexed_by_user_id),
                indexed_by_username: convert(res.indexed_by_username),
                indexed_by_username_unicode: convert(
                    res.indexed_by_username_unicode,
                ),
            })
            .unwrap()
            .into_response()
        })
        .unwrap_or_else(|err| {
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
                .into_response()
        })
}
