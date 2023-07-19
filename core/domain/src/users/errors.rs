use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug, Serialize, Deserialize)]
pub enum UsernameError {
    #[error("only ascii characters are allowed")]
    InvalidAsciiCharacters,
    #[error("use either underscores or spaces, not both")]
    UnderscoresAndSpacesNotExistsBoth,
}

#[derive(thiserror::Error, Debug, Serialize, Deserialize)]
#[error("Invalid email.")]
pub struct EmailError;

#[derive(thiserror::Error, Debug, Serialize, Deserialize)]
pub enum PasswordError {
    #[error("Failed to parse password: {0}")]
    ParseFailed(String),
    #[error("Invalid password")]
    InvalidPassword,
}

impl From<argon2::Error> for PasswordError {
    fn from(err: argon2::Error) -> Self {
        Self::ParseFailed(err.to_string())
    }
}
