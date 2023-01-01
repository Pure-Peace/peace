use tonic::{
    metadata::{Ascii, MetadataValue},
    Request, Status,
};

use crate::extensions::ClientIp;

pub fn admin_endpoints_authorization(
    req: Request<()>,
    token: MetadataValue<Ascii>,
) -> Result<Request<()>, Status> {
    match req.metadata().get("authorization") {
        Some(t) if token == t => Ok(req),
        _ => Err(Status::unauthenticated("Invalid admin token")),
    }
}

pub fn client_ip(mut request: Request<()>) -> Result<Request<()>, Status> {
    let addr = request
        .metadata()
        .get("client-ip")
        .and_then(|ip| ip.to_str().ok()?.parse().ok());
    request.extensions_mut().insert(ClientIp(addr));

    Ok(request)
}
