pub trait FromRpcClient: RpcClient {
    fn from_client(client: Self::Client) -> Self;
}

pub trait RpcClient {
    type Client;

    fn client(&self) -> Self::Client;
}

pub trait IntoService<T>: Sized + Sync + Send + 'static {
    fn into_service(self) -> T;
}

#[async_trait::async_trait]
pub trait ServiceSnapshot {
    async fn save_service_snapshot(
        &self,
        snapshot_type: peace_snapshot::SnapshotType,
        snapshot_path: &str,
    ) -> Result<(), peace_snapshot::CreateSnapshotError>;
}
