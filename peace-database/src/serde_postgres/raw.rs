use std::error::Error;
use std::ops::Deref;
use tokio_postgres::types::{FromSql, Type};

/// The raw bytes of a value, allowing "conversion" from any postgres type.
///
/// This type intentionally cannot be converted from `NULL`, and attempting to
/// do so will result in an error. Instead, use `Option<Raw>`.
pub struct Raw<'a>(pub &'a [u8]);

impl<'a> FromSql<'a> for Raw<'a> {
    fn from_sql(_ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(Raw(raw))
    }

    fn accepts(_ty: &Type) -> bool {
        true
    }
}

impl<'a> Deref for Raw<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
