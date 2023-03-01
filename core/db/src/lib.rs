pub use async_trait::async_trait;
pub use sea_orm::*;

pub mod macros;
pub mod peace;

#[async_trait]
pub trait DbConfig {
    /// Returns a configured [`ConnectOptions`]
    fn configured_opt(&self) -> ConnectOptions;

    /// Connect to database.
    async fn connect(&self) -> Result<DatabaseConnection, DbErr> {
        Database::connect(self.configured_opt()).await
    }
}
