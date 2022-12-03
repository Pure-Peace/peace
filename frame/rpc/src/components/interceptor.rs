use tonic::{
    metadata::{Ascii, MetadataValue},
    Request, Status,
};

pub fn check_auth(
    req: Request<()>,
    token: MetadataValue<Ascii>,
) -> Result<Request<()>, Status> {
    match req.metadata().get("authorization") {
        Some(t) if token == t => Ok(req),
        _ => Err(Status::unauthenticated("Invalid admin token")),
    }
}
