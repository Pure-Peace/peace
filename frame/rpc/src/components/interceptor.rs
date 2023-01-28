use std::net::IpAddr;
use tonic::{
    metadata::{Ascii, MetadataMap, MetadataValue},
    Request, Status,
};

use crate::extensions::ClientIp;

pub const X_REAL_IP: &str = "x-real-ip";
pub const X_FORWARDED_FOR: &str = "x-forwarded-for";

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
    maybe_x_real_ip(request.metadata())
        .or_else(|| maybe_x_forwarded_for(request.metadata()))
        .and_then(|addr| request.extensions_mut().insert(ClientIp(addr)));

    Ok(request)
}

fn maybe_x_forwarded_for(headers: &MetadataMap) -> Option<IpAddr> {
    headers.get(X_FORWARDED_FOR).and_then(|mv| mv.to_str().ok()).and_then(|s| {
        s.split(',').find_map(|s| s.trim().parse::<IpAddr>().ok())
    })
}

fn maybe_x_real_ip(headers: &MetadataMap) -> Option<IpAddr> {
    headers
        .get(X_REAL_IP)
        .and_then(|mv| mv.to_str().ok())
        .and_then(|s| s.parse::<IpAddr>().ok())
}
