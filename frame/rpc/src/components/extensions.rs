use std::{net::SocketAddr, ops::Deref};

#[derive(Debug, Clone)]
pub struct ClientIp(pub Option<SocketAddr>);

impl Deref for ClientIp {
    type Target = Option<SocketAddr>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
