use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::str::FromStr;
use tower_http::auth::RequireAuthorizationLayer;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::reload;

#[cfg(feature = "openapi_axum")]
use utoipa::{
    openapi::security::{self, SecurityScheme},
    IntoParams, Modify, ToSchema,
};

#[derive(Debug)]
pub enum AppError {
    ParamError(String),
    InternalError(String),
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "openapi_axum", derive(ToSchema))]
pub struct CommonResponse {
    pub status: bool,
    pub msg: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "openapi_axum", derive(IntoParams))]
pub struct ReloadLevelQuery {
    /// ```
    /// Values in: ["error", "warn", "info", "debug", "trace", "off"]
    /// ```
    pub level: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "openapi_axum", derive(IntoParams))]
pub struct DebugModeQuery {
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

/// Web api for reload [`LevelFilter`].
#[cfg_attr(feature = "openapi_axum", utoipa::path(
    get,
    path = "/admin/logs/reload_level",
    responses(
        (status = 200, description = "Reload successfully", body = [CommonResponse]),
    ),
    params(ReloadLevelQuery),
    security(("admin_token" = []))
))]
pub async fn reload_level(
    Query(query): Query<ReloadLevelQuery>,
) -> Result<Json<CommonResponse>, AppError> {
    let level = LevelFilter::from_str(&query.level)
        .map_err(|err| AppError::ParamError(err.to_string()))?;
    crate::reload_level(level)?;

    info!("<LogsApi> Reload log level to: [{}]", level);
    Ok(Json(CommonResponse { status: true, msg: None }))
}

/// Web api for set log debug mode.
#[cfg_attr(feature = "openapi_axum", utoipa::path(
    get,
    path = "/admin/logs/debug_mode",
    responses(
        (status = 200, description = "Debug mode toggle successfully", body = [CommonResponse]),
    ),
    params(DebugModeQuery),
    security(("admin_token" = []))
))]
pub async fn debug_mode(
    Query(query): Query<DebugModeQuery>,
) -> Result<Json<CommonResponse>, AppError> {
    crate::debug_mode(query.enabled)?;

    info!("<LogsRpc> Toggle debug mode: [{}]", query.enabled);
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
    use crate::api::{
        debug_mode, reload_level, DebugModeQuery, ReloadLevelQuery,
    };
    use axum::extract::Query;

    #[tokio::test]
    async fn try_reload_level() {
        assert!(reload_level(Query(ReloadLevelQuery {
            level: "info".to_string()
        }))
        .await
        .is_ok());
        assert!(reload_level(Query(ReloadLevelQuery {
            level: "3".to_string()
        }))
        .await
        .is_ok());
        assert!(reload_level(Query(ReloadLevelQuery {
            level: "xxx".to_string()
        }))
        .await
        .is_err());
    }

    #[tokio::test]
    async fn try_debug_mode() {
        assert!(debug_mode(Query(DebugModeQuery { enabled: true }))
            .await
            .is_ok());
        assert!(debug_mode(Query(DebugModeQuery { enabled: false }))
            .await
            .is_ok());
    }
}
