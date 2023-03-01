pub mod docs;
pub mod error;
pub mod extractors;
pub mod parser;
pub mod repository;
pub mod routes;
pub mod service;

pub use docs::*;
pub use error::*;
pub use service::*;

pub const CHO_PROTOCOL: (&str, &str) = ("cho-protocol", "19");
pub const CHO_TOKEN: &str = "cho-token";
pub const X_REAL_IP: &str = "x-real-ip";
