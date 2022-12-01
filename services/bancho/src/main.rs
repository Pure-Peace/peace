use bancho::rpc::Bancho;

use peace_pb::services::bancho::bancho_rpc_server::BanchoRpcServer;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let svc = Bancho::default();

    Server::builder()
        .add_service(BanchoRpcServer::new(svc))
        .serve(addr)
        .await?;

    Ok(())
}
