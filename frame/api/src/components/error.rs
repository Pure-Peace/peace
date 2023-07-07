use axum::{
    http::{header::WWW_AUTHENTICATE, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use std::{borrow::Cow, collections::HashMap};

/// A common error type that can be used throughout the API.
///
/// Can be returned in a `Result` from an API handler function.
///
/// For convenience, this represents both API errors as well as internal
/// recoverable errors, and maps them to appropriate status codes along with at
/// least a minimally useful error message in a plain text body, or a JSON body
/// in the case of `UnprocessableEntity`.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Return `401 Unauthorized`
    #[error("authentication required")]
    Unauthorized,

    /// Return `403 Forbidden`
    #[error("user may not perform that action")]
    Forbidden,

    /// Return `404 Not Found`
    #[error("request path not found")]
    NotFound,

    /// Return `408 Request Timeout`
    #[error("request timed out")]
    Timeout,

    /// Return `422 Unprocessable Entity`
    #[error("error in the request body")]
    UnprocessableEntity {
        errors: HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>,
    },

    /// Return `503 Service Unavailable
    #[error("service is overloaded, try again later")]
    Unavailable,

    /// Return `500 Internal Server Error`
    #[error("an internal server error occurred")]
    Internal,

    /// Return `500 Internal Server Error` on a `anyhow::Error`.
    #[error("an internal server error occurred")]
    RpcError(anyhow::Error),

    /// Return `500 Internal Server Error` on a `anyhow::Error`.
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl Error {
    /// Convenient constructor for `Error::UnprocessableEntity`.
    ///
    /// Multiple for the same key are collected into a list for that key.
    pub fn unprocessable_entity<K, V>(
        errors: impl IntoIterator<Item = (K, V)>,
    ) -> Self
    where
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        let mut error_map = HashMap::new();

        for (key, val) in errors {
            error_map
                .entry(key.into())
                .or_insert_with(Vec::new)
                .push(val.into());
        }

        Self::UnprocessableEntity { errors: error_map }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Timeout => StatusCode::REQUEST_TIMEOUT,
            Self::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::UnprocessableEntity { .. } => {
                StatusCode::UNPROCESSABLE_ENTITY
            },
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            Self::RpcError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<Error> for Response {
    fn from(err: Error) -> Self {
        err.into_response()
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Self::UnprocessableEntity { errors } => {
                #[derive(serde::Serialize)]
                struct Errors {
                    errors: HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>,
                }

                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(Errors { errors }),
                )
                    .into_response();
            },
            Self::Unauthorized => {
                return (
                    self.status_code(),
                    // Include the `WWW-Authenticate` challenge required in the
                    // specification for the `401
                    // Unauthorized` response code: https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/401
                    [(WWW_AUTHENTICATE, HeaderValue::from_static("Token"))]
                        .into_iter()
                        .collect::<HeaderMap>(),
                    self.to_string(),
                )
                    .into_response();
            },

            Self::RpcError(ref e) => {
                error!("[RPC error]: {}", e)
            },
            // Other errors get mapped normally.
            _ => (),
        }

        (self.status_code(), self.to_string()).into_response()
    }
}

pub fn map_err(err: impl std::fmt::Display) -> crate::error::Error {
    Error::Anyhow(anyhow!("{}", err))
}
