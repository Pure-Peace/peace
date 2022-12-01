use peace_db::rpc::Db;
use peace_pb::services::db::db_rpc_server::DbRpcServer;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let greeter = Db::default();

    Server::builder()
        .add_service(DbRpcServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
