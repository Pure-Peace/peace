#![allow(dead_code)]
use std::str::FromStr;

use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};

use tower_http::auth::RequireAuthorizationLayer;

use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::reload;

#[derive(Debug)]
pub enum AppError {
    ParamError(String),
    InternalError(String),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CommonResponse {
    pub status: bool,
    pub msg: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReloadLevel {
    pub level: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DebugMode {
    pub enabled: bool,
}

/// Web api for reload [`LevelFilter`].
pub async fn reload_level(
    Query(params): Query<ReloadLevel>,
) -> Result<Json<CommonResponse>, AppError> {
    let level = LevelFilter::from_str(&params.level)
        .map_err(|err| AppError::ParamError(err.to_string()))?;
    crate::reload_level(level)?;

    info!("<LogsApi> Reload log level to: [{}]", level);
    Ok(Json(CommonResponse { status: true, msg: None }))
}

/// Web api for set log debug mode.
pub async fn debug_mode(
    Query(params): Query<DebugMode>,
) -> Result<Json<CommonResponse>, AppError> {
    crate::debug_mode(params.enabled)?;

    info!("<LogsRpc> Toggle debug mode: [{}]", params.enabled);
    Ok(Json(CommonResponse { status: true, msg: None }))
}

/// Admin routers [`Router`]
///
///
/// [`reload_level`] : `GET` `/admin/logs/reload_level`
///
/// [`debug_mode`] : `GET` `/admin/logs/debug_mode`
///
pub fn admin_routers(admin_token: Option<&str>) -> Router {
    let router = Router::new()
        .route("/admin/logs/reload_level", get(reload_level))
        .route("/admin/logs/debug_mode", get(debug_mode));

    if let Some(token) = &admin_token {
        router.layer(RequireAuthorizationLayer::bearer(token))
    } else {
        router
    }
}

impl From<reload::Error> for AppError {
    fn from(inner: reload::Error) -> Self {
        AppError::InternalError(inner.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::ParamError(err) => (StatusCode::BAD_REQUEST, err),
            AppError::InternalError(err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, err)
            },
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

#[cfg(test)]
mod test {
    use crate::api::{debug_mode, reload_level, DebugMode, ReloadLevel};
    use axum::extract::Query;

    #[tokio::test]
    async fn try_reload_level() {
        assert!(reload_level(Query(ReloadLevel { level: "info".to_string() }))
            .await
            .is_ok());
        assert!(reload_level(Query(ReloadLevel { level: "3".to_string() }))
            .await
            .is_ok());
        assert!(reload_level(Query(ReloadLevel { level: "xxx".to_string() }))
            .await
            .is_err());
    }

    #[tokio::test]
    async fn try_debug_mode() {
        assert!(debug_mode(Query(DebugMode { enabled: true })).await.is_ok());
        assert!(debug_mode(Query(DebugMode { enabled: false })).await.is_ok());
    }
}
