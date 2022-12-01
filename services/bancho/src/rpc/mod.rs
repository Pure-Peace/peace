use peace_pb::services::bancho::{
    bancho_rpc_server::BanchoRpc, HelloReply, HelloRequest,
};
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct Bancho {}

#[tonic::async_trait]
impl BanchoRpc for Bancho {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name).into(),
        };

        Ok(Response::new(reply))
    }
}
