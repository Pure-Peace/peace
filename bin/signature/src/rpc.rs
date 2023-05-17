use peace_pb::{
    base::ExecSuccess,
    signature::{
        signature_rpc_server, GetPublicKeyRequest, GetPublicKeyResponse,
        ReloadFromPemFileRequest, ReloadFromPemRequest, SignMessageRequest,
        SignMessageResponse, VerifyMessageRequest, VerifyMessageResponse,
    },
};
use peace_services::signature::DynSignatureService;
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct SignatureRpcImpl {
    pub signature_service: DynSignatureService,
}

impl SignatureRpcImpl {
    pub fn new(signature_service: DynSignatureService) -> Self {
        Self { signature_service }
    }
}

#[tonic::async_trait]
impl signature_rpc_server::SignatureRpc for SignatureRpcImpl {
    async fn sign_message(
        &self,
        request: Request<SignMessageRequest>,
    ) -> Result<Response<SignMessageResponse>, Status> {
        self.signature_service
            .sign(request.into_inner().message.into())
            .await
            .map_err(Status::from)
            .map(|signature| Response::new(SignMessageResponse { signature }))
    }

    async fn verify_message(
        &self,
        request: Request<VerifyMessageRequest>,
    ) -> Result<Response<VerifyMessageResponse>, Status> {
        let VerifyMessageRequest { message, signature } = request.into_inner();

        self.signature_service
            .verify(message.into(), signature.into())
            .await
            .map_err(Status::from)
            .map(|is_valid| Response::new(VerifyMessageResponse { is_valid }))
    }

    async fn reload_from_pem(
        &self,
        request: Request<ReloadFromPemRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        self.signature_service
            .reload_from_pem(request.into_inner().ed25519_private_key.into())
            .await
            .map_err(Status::from)?;

        Ok(Response::new(ExecSuccess::default()))
    }

    async fn reload_from_pem_file(
        &self,
        request: Request<ReloadFromPemFileRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        self.signature_service
            .reload_from_pem_file(
                request.into_inner().ed25519_private_key_file_path.into(),
            )
            .await
            .map_err(Status::from)?;

        Ok(Response::new(ExecSuccess::default()))
    }

    async fn get_public_key(
        &self,
        _: Request<GetPublicKeyRequest>,
    ) -> Result<Response<GetPublicKeyResponse>, Status> {
        self.signature_service.public_key().await.map_err(Status::from).map(
            |public_key| Response::new(GetPublicKeyResponse { public_key }),
        )
    }
}
