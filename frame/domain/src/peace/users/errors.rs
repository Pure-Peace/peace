#[derive(thiserror::Error, Debug)]
#[error("Invalid username.")]
pub struct UsernameError;

#[derive(thiserror::Error, Debug)]
#[error("Invalid email.")]
pub struct EmailError;

#[derive(thiserror::Error, Debug)]
#[error("Failed to process password.")]
pub struct PasswordError {
    #[from]
    source: argon2::Error,
}
