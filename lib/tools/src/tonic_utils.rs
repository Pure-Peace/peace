use crate::constants::X_REAL_IP;

/// A wrapper around tonic::Request that provides additional functionality.
#[derive(Debug)]
pub struct RpcRequest<T>(tonic::Request<T>);

impl<T> RpcRequest<T> {
    /// Creates a new RpcRequest with the given payload.
    pub fn new(raw: T) -> Self {
        Self(tonic::Request::new(raw))
    }

    /// Creates a new RpcRequest from an existing tonic::Request.
    pub fn from_request(request: tonic::Request<T>) -> Self {
        Self(request)
    }

    /// Converts the RpcRequest into a tonic::Request.
    pub fn to_request(self) -> tonic::Request<T> {
        self.0
    }

    /// Sets the X-Real-IP header of the RpcRequest to the specified IP address.
    ///
    /// # Arguments
    ///
    /// * `ip` - A string representation of the IP address to set as the X-Real-IP header.
    ///
    /// # Returns
    ///
    /// The modified RpcRequest.
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
    /// Creates an RpcRequest from a tonic::Request.
    fn from(request: tonic::Request<T>) -> Self {
        Self(request)
    }
}

impl<T> Into<tonic::Request<T>> for RpcRequest<T> {
    /// Converts an RpcRequest into a tonic::Request.
    fn into(self) -> tonic::Request<T> {
        self.0
    }
}

impl<T> std::ops::Deref for RpcRequest<T> {
    type Target = tonic::Request<T>;

    /// Dereferences the RpcRequest to a tonic::Request.
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for RpcRequest<T> {
    /// Mutably dereferences the RpcRequest to a tonic::Request.
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
