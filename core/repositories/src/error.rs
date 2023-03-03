use peace_db::DbErr;

#[derive(thiserror::Error, Debug)]
pub enum GetUserError {
    #[error("user not exists")]
    UserNotExists,
    #[error("invalid login data")]
    DbErr(#[from] DbErr),
}
