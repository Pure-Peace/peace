use async_trait::async_trait;
use peace_rpc_error::{RpcError, TonicError};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;
use tonic::Status;

#[derive(thiserror::Error, Debug, Serialize, Deserialize, RpcError)]
pub enum LoadSnapshotError {
    #[error("failed to deserialize snapshot: {0}")]
    DeserializeError(String),
    #[error("failed to read file: {0}")]
    ReadFileError(String),
    #[error("tonic error: {0}")]
    TonicError(String),
    #[error("error: {0}")]
    AnyError(String),
}

impl TonicError for LoadSnapshotError {
    fn tonic_error(s: Status) -> Self {
        Self::TonicError(s.message().to_owned())
    }
}

#[derive(thiserror::Error, Debug, Serialize, Deserialize, RpcError)]
pub enum CreateSnapshotError {
    #[error("failed to serialize snapshot: {0}")]
    SerializeError(String),
    #[error("failed to create directory: {0}")]
    CreateDirError(String),
    #[error("failed to write file: {0}")]
    WriteFileError(String),
    #[error("tonic error: {0}")]
    TonicError(String),
    #[error("error: {0}")]
    AnyError(String),
}

impl TonicError for CreateSnapshotError {
    fn tonic_error(s: Status) -> Self {
        Self::TonicError(s.message().to_owned())
    }
}

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum,
)]
#[serde(rename_all = "lowercase")]
pub enum SnapshotType {
    Binary,
    Json,
}

pub trait SnapshotTime {
    fn snapshot_time(&self) -> u64;
}

pub trait SnapshotConfig {
    fn snapshot_path(&self) -> &str;
    fn snapshot_type(&self) -> SnapshotType;
    fn should_save_snapshot(&self) -> bool;
    fn should_load_snapshot(&self) -> bool;
    fn snapshot_expired_secs(&self) -> u64;
}

#[async_trait]
pub trait CreateSnapshot<D> {
    async fn create_snapshot(&self) -> D;
}

#[async_trait]
pub trait LoadSnapshotFrom: Sized {
    async fn load_snapshot_from(
        snapshot_type: SnapshotType,
        snapshot_path: &str,
    ) -> Result<Self, LoadSnapshotError>;
}

#[async_trait]
impl<T> LoadSnapshotFrom for T
where
    T: for<'a> serde::Deserialize<'a>,
{
    async fn load_snapshot_from(
        snapshot_type: SnapshotType,
        snapshot_path: &str,
    ) -> Result<Self, LoadSnapshotError> {
        let content = fs::read(snapshot_path)
            .await
            .map_err(|err| LoadSnapshotError::ReadFileError(err.to_string()))?;

        Ok(match snapshot_type {
            SnapshotType::Binary => {
                bincode::deserialize(&content).map_err(|err| {
                    LoadSnapshotError::DeserializeError(err.to_string())
                })?
            },
            SnapshotType::Json => {
                serde_json::from_slice(&content).map_err(|err| {
                    LoadSnapshotError::DeserializeError(err.to_string())
                })?
            },
        })
    }
}

pub trait SnapshotExpired {
    fn snapshot_expired(&self, expires: u64) -> bool;
}

impl<T> SnapshotExpired for T
where
    T: SnapshotTime,
{
    fn snapshot_expired(&self, expires: u64) -> bool {
        (self.snapshot_time() + expires)
            < std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time before Unix epoch")
                .as_secs()
    }
}

#[async_trait]
pub trait SaveSnapshotTo<D> {
    async fn save_snapshot_to(
        &self,
        snapshot_type: SnapshotType,
        snapshot_path: &str,
    ) -> Result<usize, CreateSnapshotError>;
}

#[async_trait]
impl<T, D> SaveSnapshotTo<D> for T
where
    T: CreateSnapshot<D> + Sync + Send,
    D: serde::Serialize + Send,
{
    async fn save_snapshot_to(
        &self,
        snapshot_type: SnapshotType,
        snapshot_path: &str,
    ) -> Result<usize, CreateSnapshotError> {
        let create_snapshot = self.create_snapshot().await;

        let bytes_data = match snapshot_type {
            SnapshotType::Binary => bincode::serialize(&create_snapshot)
                .map_err(|err| err.to_string()),
            SnapshotType::Json => serde_json::to_vec(&create_snapshot)
                .map_err(|err| err.to_string()),
        }
        .map_err(CreateSnapshotError::SerializeError)?;

        let path = Path::new(snapshot_path);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.map_err(|err| {
                CreateSnapshotError::CreateDirError(err.to_string())
            })?;
        }

        fs::write(path, &bytes_data).await.map_err(|err| {
            CreateSnapshotError::WriteFileError(err.to_string())
        })?;

        Ok(bytes_data.len())
    }
}
