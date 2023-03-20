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
    #[error("{}", .0.message())]
    RpcError(#[from] Status),
}

impl GeoipError {
    fn tonic_code(&self) -> Code {
        Code::Internal
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl IntoResponse for GeoipError {
    fn into_response(self) -> Response {
        (self.status_code(), self.to_string()).into_response()
    }
}

impl From<GeoipError> for Status {
    fn from(err: GeoipError) -> Self {
        Status::new(err.tonic_code(), err.to_string())
    }
}
