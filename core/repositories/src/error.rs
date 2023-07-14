use peace_db::DbErr;

#[derive(thiserror::Error, Debug, Serialize, Deserialize)]
pub enum GetUserError {
    #[error("user not exists")]
    UserNotExists,
    #[error("database err: {0}")]
    DbErr(String),
}

impl From<DbErr> for GetUserError {
    fn from(err: DbErr) -> Self {
        Self::DbErr(err.to_string())
    }
}
