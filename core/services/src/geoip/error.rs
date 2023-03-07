use axum::response::{IntoResponse, Response};
use hyper::StatusCode;
use maxminddb::MaxMindDBError;
use tonic::{Code, Status};

#[derive(thiserror::Error, Debug)]
pub enum GeoipError {
    #[error("geo-ip local was not initialized")]
    NotInitialized,
    #[error("failed to lookup ip address: {0}")]
    LookupError(#[source] MaxMindDBError),
    #[error("this only for local service")]
    OnlyLocalService,
    #[error("failed to load geo-ip database: {0}")]
    FailedToLoadDatabase(#[source] MaxMindDBError),
    #[error("{0}")]
    RpcError(#[from] Status),
}

impl GeoipError {
    fn tonic_code(&self) -> Code {
        match self {
            _ => Code::Internal,
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for GeoipError {
    fn into_response(self) -> Response {
        match self {
            _ => (self.status_code(), self.to_string()).into_response(),
        }
    }
}

impl Into<Status> for GeoipError {
    fn into(self) -> Status {
        match self {
            _ => Status::new(self.tonic_code(), self.to_string()),
        }
    }
}
