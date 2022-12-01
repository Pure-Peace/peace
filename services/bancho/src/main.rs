use bancho::rpc::Bancho;

use peace_pb::services::bancho::{
    bancho_rpc_server::BanchoRpcServer, BANCHO_DESCRIPTOR_SET,
};
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let svc = Bancho::default();
    let reflect_svc = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(BANCHO_DESCRIPTOR_SET)
        .build()
        .unwrap();

    Server::builder()
        .add_service(reflect_svc)
        .add_service(BanchoRpcServer::new(svc))
        .serve(addr)
        .await?;

    Ok(())
}
