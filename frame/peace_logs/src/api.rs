use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_http::auth::RequireAuthorizationLayer;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::reload;

#[cfg(feature = "openapi_axum")]
use utoipa::{
    openapi::security::{self, SecurityScheme},
    IntoParams, Modify, ToSchema,
};

use crate::LogLevel;

#[derive(Debug)]
pub enum AppError {
    ParamError(String),
    InternalError(String),
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "openapi_axum", derive(ToSchema))]
pub struct CommonHandleResponse {
    pub success: bool,
    pub msg: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "openapi_axum", derive(IntoParams))]
pub struct SetLogLevelParam {
    /// ```
    /// `level` in: ["error", "warn", "info", "debug", "trace", "off"]
    /// ```
    #[param(inline, value_type = LogLevel)]
    pub level: LogLevel,
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "openapi_axum", derive(IntoParams))]
pub struct ToggleDebugModeParam {
    pub enabled: bool,
}

#[cfg(feature = "openapi_axum")]
pub struct AdminAuth;

#[cfg(feature = "openapi_axum")]
impl Modify for AdminAuth {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "admin_token",
                SecurityScheme::Http(security::Http::new(
                    security::HttpAuthScheme::Bearer,
                )),
            )
        }
    }
}

/// Set the global log level.
#[cfg_attr(feature = "openapi_axum", utoipa::path(
    put,
    context_path = "/admin",
    path = "/logs/set_level/{level}",
    tag = "admin",
    responses(
        (status = 200, description = "Success", body = [CommonHandleResponse]),
    ),
    params(SetLogLevelParam),
    security(("admin_token" = []))
))]
pub async fn set_level(
    Path(param): Path<LogLevel>,
) -> Result<Json<CommonHandleResponse>, AppError> {
    let level = LevelFilter::from(param);
    crate::set_level(level)?;

    info!("<LogsApi> Reload log level to: [{}]", level);
    Ok(Json(CommonHandleResponse { success: true, msg: None }))
}

/// Set the log env filter.
#[cfg_attr(feature = "openapi_axum", utoipa::path(
    put,
    context_path = "/admin",
    path = "/logs/set_env_filter/{filter}",
    tag = "admin",
    responses(
        (status = 200, description = "Success", body = [CommonHandleResponse]),
    ),
    params(
        ("filter" = String, Path, description = "env filter string", example = "peace_logs::api=info")
    ),
    security(("admin_token" = []))
))]
pub async fn set_env_filter(
    Path(filter): Path<String>,
) -> Result<Json<CommonHandleResponse>, AppError> {
    crate::set_env_filter(&filter)?;

    info!("<LogsApi> Set env filter to: [{}]", filter);
    Ok(Json(CommonHandleResponse { success: true, msg: None }))
}

/// Get current log configs.
#[cfg_attr(feature = "openapi_axum", utoipa::path(
    get,
    context_path = "/admin",
    path = "/logs/config",
    tag = "admin",
    responses(
        (status = 200, description = "Success", body = [String]),
    ),
    security(("admin_token" = []))
))]
pub async fn config() -> Result<String, AppError> {
    Ok(crate::env_filter(None).to_string())
}

/// Toggle debug mode.
///
/// Turning on debug will display information such as code line number, source file, thread id, etc.
#[cfg_attr(feature = "openapi_axum", utoipa::path(
    put,
    context_path = "/admin",
    path = "/logs/debug_mode/{enabled}",
    tag = "admin",
    responses(
        (status = 200, description = "Debug mode toggle successfully", body = [CommonHandleResponse]),
    ),
    params(ToggleDebugModeParam),
    security(("admin_token" = []))
))]
pub async fn debug_mode(
    Path(param): Path<ToggleDebugModeParam>,
) -> Result<Json<CommonHandleResponse>, AppError> {
    crate::toggle_debug_mode(param.enabled)?;

    info!("<LogsApi> Toggle debug mode: [{}]", param.enabled);
    Ok(Json(CommonHandleResponse { success: true, msg: None }))
}

/// Admin routers [`Router`]
///
///
/// [`set_level`] : `PUT` `/admin/logs/set_level/:level`
/// [`set_env_filter`] : `PUT` `/admin/logs/set_env_filter/:filter`
/// [`debug_mode`] : `PUT` `/admin/logs/debug_mode/:enabled`
///
pub fn admin_routers(admin_token: Option<&str>) -> Router {
    let router = Router::new()
        .route("/admin/logs/set_level/:level", put(set_level))
        .route("/admin/logs/set_env_filter/:filter", put(set_env_filter))
        .route("/admin/logs/config", get(config))
        .route("/admin/logs/debug_mode/:enabled", put(debug_mode));

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
    use crate::{
        api::{debug_mode, set_level, ToggleDebugModeParam},
        LogLevel,
    };
    use axum::extract::Path;

    #[tokio::test]
    async fn try_set_level() {
        assert!(set_level(Path(LogLevel::Debug)).await.is_ok());
    }

    #[tokio::test]
    async fn try_debug_mode() {
        assert!(debug_mode(Path(ToggleDebugModeParam { enabled: true }))
            .await
            .is_ok());
        assert!(debug_mode(Path(ToggleDebugModeParam { enabled: false }))
            .await
            .is_ok());
    }
}
