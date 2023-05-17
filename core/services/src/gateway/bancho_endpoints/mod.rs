pub mod components;
pub mod docs;
pub mod error;
pub mod extractors;
pub mod parser;
pub mod routes;
pub mod services;

pub use docs::*;
pub use error::*;
pub use services::*;

pub const CHO_PROTOCOL: (&str, &str) = ("cho-protocol", "19");
pub const CHO_TOKEN: &str = "cho-token";
pub const X_REAL_IP: &str = "x-real-ip";
