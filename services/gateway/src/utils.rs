use std::ops::{Deref, DerefMut};

use crate::bancho::constants::X_REAL_IP;



pub fn map_rpc_err(err: tonic::Status) -> peace_api::error::Error {
    error!("RPC call failed with: {}", err);
    peace_api::error::Error::Internal
}
