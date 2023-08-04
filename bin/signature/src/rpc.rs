use core_signature::DynSignatureService;
use pb_base::ExecSuccess;
use pb_signature::{
    signature_rpc_server, GetPublicKeyRequest, GetPublicKeyResponse,
    ReloadFromPemFileRequest, ReloadFromPemRequest, SignMessageRequest,
    SignMessageResponse, VerifyMessageRequest, VerifyMessageResponse,
};
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
        let signature = self
            .signature_service
            .sign(request.into_inner().message.into())
            .await?;

        Ok(Response::new(SignMessageResponse { signature }))
    }

    async fn verify_message(
        &self,
        request: Request<VerifyMessageRequest>,
    ) -> Result<Response<VerifyMessageResponse>, Status> {
        let VerifyMessageRequest { message, signature } = request.into_inner();

        let is_valid = self
            .signature_service
            .verify(message.into(), signature.into())
            .await?;

        Ok(Response::new(VerifyMessageResponse { is_valid }))
    }

    async fn reload_from_pem(
        &self,
        request: Request<ReloadFromPemRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        let res = self
            .signature_service
            .reload_from_pem(request.into_inner().ed25519_private_key.into())
            .await?;

        Ok(Response::new(res))
    }

    async fn reload_from_pem_file(
        &self,
        request: Request<ReloadFromPemFileRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        let res = self
            .signature_service
            .reload_from_pem_file(
                request.into_inner().ed25519_private_key_file_path.into(),
            )
            .await?;

        Ok(Response::new(res))
    }

    async fn get_public_key(
        &self,
        _: Request<GetPublicKeyRequest>,
    ) -> Result<Response<GetPublicKeyResponse>, Status> {
        let public_key = self.signature_service.public_key().await?;

        Ok(Response::new(GetPublicKeyResponse { public_key }))
    }
}
