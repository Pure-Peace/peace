use std::net::IpAddr;
use tonic::{
    metadata::{Ascii, MetadataMap, MetadataValue},
    Request, Status,
};

use crate::extensions::ClientIp;

pub const X_REAL_IP: &str = "x-real-ip";
pub const X_FORWARDED_FOR: &str = "x-forwarded-for";

// Authorization middleware for admin endpoints
pub fn admin_endpoints_authorization(
    req: Request<()>,
    token: MetadataValue<Ascii>,
) -> Result<Request<()>, Status> {
    // Check if the `authorization` header is present in the request metadata
    match req.metadata().get("authorization") {
        // If it is, check if the token in the header matches the token passed as an argument
        Some(t) if token == t => Ok(req),
        // If the header is not present or the tokens don't match, return an error
        _ => Err(Status::unauthenticated("Invalid admin token")),
    }
}

// Middleware that extracts the client IP address from the request metadata
pub fn client_ip(mut request: Request<()>) -> Result<Request<()>, Status> {
    // Try to extract the IP address from the `X-Real-IP` or `X-Forwarded-For` headers
    maybe_x_real_ip(request.metadata())
        .or_else(|| maybe_x_forwarded_for(request.metadata()))
        .and_then(|addr| request.extensions_mut().insert(ClientIp(addr)));

    Ok(request)
}

// Helper function that extracts an IP address from the `X-Forwarded-For` header
fn maybe_x_forwarded_for(headers: &MetadataMap) -> Option<IpAddr> {
    headers
        .get(X_FORWARDED_FOR)
        .and_then(|mv| mv.to_str().ok()) // Convert the header value to a string
        .and_then(|s| {
            s.split(',') // Split the value into a list of IP addresses
                .find_map(|s| s.trim().parse::<IpAddr>().ok()) // Find the first valid IP address
        })
}

// Helper function that extracts an IP address from the `X-Real-IP` header
fn maybe_x_real_ip(headers: &MetadataMap) -> Option<IpAddr> {
    headers
        .get(X_REAL_IP)
        .and_then(|mv| mv.to_str().ok()) // Convert the header value to a string
        .and_then(|s| s.parse::<IpAddr>().ok()) // Parse the IP address from the string
}
