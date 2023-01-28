use std::{net::IpAddr, ops::Deref};
use tonic::{Request, Status};

#[derive(Debug, Clone)]
pub struct ClientIp(pub IpAddr);

impl ClientIp {
    pub fn from_request<T>(request: &Request<T>) -> Result<Self, Status> {
        Ok(request
            .extensions()
            .get::<ClientIp>()
            .ok_or(Status::internal("No client ip"))?
            .to_owned())
    }
}

impl Deref for ClientIp {
    type Target = IpAddr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
