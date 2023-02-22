#[derive(thiserror::Error, Debug)]
pub enum UsernameError {
    #[error("only ascii characters are allowed.")]
    InvalidAsciiCharacters,
    #[error("use either underscores or spaces, not both.")]
    UnderscoresAndSpacesNotExistsBoth,
}

#[derive(thiserror::Error, Debug)]
#[error("Invalid email.")]
pub struct EmailError;

#[derive(thiserror::Error, Debug)]
#[error("Failed to process password.")]
pub struct PasswordError {
    #[from]
    source: argon2::Error,
}
