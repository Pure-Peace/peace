use peace_pb::services::db::{db_rpc_server::DbRpc, HelloReply, HelloRequest};
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct Db {}

#[tonic::async_trait]
impl DbRpc for Db {
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
