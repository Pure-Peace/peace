use std::net::IpAddr;
use tonic::Request;

use crate::constants::X_REAL_IP;

pub struct RawRequest;

impl RawRequest {
    pub fn add_client_ip<T>(raw: T, client_ip: IpAddr) -> Request<T> {
        let mut req = Request::new(raw);
        req.metadata_mut().insert(
            X_REAL_IP,
            client_ip
                .to_string()
                .parse()
                .expect("IpAddr to string err: should never happened"),
        );
        req
    }
}
