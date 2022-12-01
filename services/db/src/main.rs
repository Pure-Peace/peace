use peace_db::rpc::Db;
use peace_pb::services::db::{db_rpc_server::DbRpcServer, DB_DESCRIPTOR_SET};
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let svc = Db::default();
    let reflect_svc = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(DB_DESCRIPTOR_SET)
        .build()
        .unwrap();

    Server::builder()
        .add_service(reflect_svc)
        .add_service(DbRpcServer::new(svc))
        .serve(addr)
        .await?;

    Ok(())
}
