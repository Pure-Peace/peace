use peace_pb::services::peace_db::{
    peace_db_rpc_server::PeaceDbRpc, HelloReply, HelloRequest,
};
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct PeaceDbService {}

#[tonic::async_trait]
impl PeaceDbRpc for PeaceDbService {
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
