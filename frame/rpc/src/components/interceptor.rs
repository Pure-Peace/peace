use tonic::{
    metadata::{Ascii, MetadataValue},
    Request, Status,
};

use crate::extensions::ClientIp;

pub const CLIENT_IP_HEADER: &str = "client-ip";

pub fn admin_endpoints_authorization(
    req: Request<()>,
    token: MetadataValue<Ascii>,
) -> Result<Request<()>, Status> {
    match req.metadata().get("authorization") {
        Some(t) if token == t => Ok(req),
        _ => Err(Status::unauthenticated("Invalid admin token")),
    }
}

pub fn client_ip(
    mut request: Request<()>,
) -> Result<Request<()>, Status> {
    let addr = request
        .metadata()
        .get(CLIENT_IP_HEADER)
        .and_then(|ip| ip.to_str().ok()?.parse().ok())
        .ok_or_else(|| Status::internal("client-ip header is required"))?;

    request.extensions_mut().insert(ClientIp(addr));

    Ok(request)
}
