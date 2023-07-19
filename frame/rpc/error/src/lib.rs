pub use peace_rpc_error_derive::*;

use serde::{Deserialize, Serialize};
use std::fmt::Display;

pub const SERVICE_ERROR_HEADER: &str = "x-peace-service-error";

pub trait RpcError<'de>: Serialize + Deserialize<'de> + Display {}

pub trait TonicError {
    fn tonic_error(s: tonic::Status) -> Self;
}
