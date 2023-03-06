use maxminddb::MaxMindDBError;

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
}
