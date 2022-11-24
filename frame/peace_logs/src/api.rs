use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::put,
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
pub struct CommonResponse {
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

/// Set the log level.
#[cfg_attr(feature = "openapi_axum", utoipa::path(
    put,
    context_path = "/admin",
    path = "/logs/set_level/{level}",
    tag = "admin",
    responses(
        (status = 200, description = "Reload successfully", body = [CommonResponse]),
    ),
    params(SetLogLevelParam),
    security(("admin_token" = []))
))]
pub async fn set_level(
    Path(param): Path<LogLevel>,
) -> Result<Json<CommonResponse>, AppError> {
    let level = LevelFilter::from(param);
    crate::set_level(level)?;

    info!("<LogsApi> Reload log level to: [{}]", level);
    Ok(Json(CommonResponse { success: true, msg: None }))
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
        (status = 200, description = "Debug mode toggle successfully", body = [CommonResponse]),
    ),
    params(ToggleDebugModeParam),
    security(("admin_token" = []))
))]
pub async fn debug_mode(
    Path(param): Path<ToggleDebugModeParam>,
) -> Result<Json<CommonResponse>, AppError> {
    crate::toggle_debug_mode(param.enabled)?;

    info!("<LogsApi> Toggle debug mode: [{}]", param.enabled);
    Ok(Json(CommonResponse { success: true, msg: None }))
}

/// Admin routers [`Router`]
///
///
/// [`set_level`] : `PUT` `/admin/logs/set_level/:level`
///
/// [`debug_mode`] : `PUT` `/admin/logs/debug_mode/:enabled`
///
pub fn admin_routers(admin_token: Option<&str>) -> Router {
    let router = Router::new()
        .route("/admin/logs/set_level/:level", put(set_level))
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
