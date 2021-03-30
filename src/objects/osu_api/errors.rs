#[allow(unused_imports)]
use super::depends::*;

#[derive(Debug, Eq, PartialEq)]
pub enum ApiError {
    NotExists,
    RequestError,
    ParseError,
}
