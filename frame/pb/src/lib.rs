pub mod frame;
pub mod services;

#[derive(Debug, Clone)]
pub struct ConvertError(String);

impl ConvertError {
    pub fn new<D: ::core::fmt::Display>(raw: D) -> Self {
        Self(raw.to_string())
    }
}
