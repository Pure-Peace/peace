use crate::constants::X_REAL_IP;

#[derive(Debug)]
pub struct RpcRequest<T>(tonic::Request<T>);

impl<T> RpcRequest<T> {
    pub fn new(raw: T) -> Self {
        Self(tonic::Request::new(raw))
    }

    pub fn from_request(request: tonic::Request<T>) -> Self {
        Self(request)
    }

    pub fn to_request(self) -> tonic::Request<T> {
        self.0
    }

    pub fn client_ip_header(mut self, ip: impl ToString) -> Self {
        self.metadata_mut().insert(
            X_REAL_IP,
            ip.to_string()
                .parse()
                .expect("IpAddr to string err: should never happened"),
        );
        self
    }
}

impl<T> From<tonic::Request<T>> for RpcRequest<T> {
    fn from(request: tonic::Request<T>) -> Self {
        Self(request)
    }
}

impl<T> Into<tonic::Request<T>> for RpcRequest<T> {
    fn into(self) -> tonic::Request<T> {
        self.0
    }
}

impl<T> std::ops::Deref for RpcRequest<T> {
    type Target = tonic::Request<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for RpcRequest<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
