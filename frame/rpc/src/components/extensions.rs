use std::{net::IpAddr, ops::Deref};

#[derive(Debug, Clone)]
pub struct ClientIp(pub IpAddr);

impl Deref for ClientIp {
    type Target = IpAddr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
