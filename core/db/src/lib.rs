pub use async_trait::async_trait;
pub use sea_orm::*;

use std::ops::Deref;

pub mod macros;
pub mod peace;

#[async_trait]
pub trait DbConfig<T>
where
    T: Default,
{
    /// Returns a configured [`ConnectOptions`]
    fn configured_opt(&self) -> ConnectOptions;

    /// Connect to database.
    async fn connect(&self) -> Result<DbConnection<T>, DbErr> {
        Database::connect(self.configured_opt())
            .await
            .map(|conn| DbConnection { conn, mark: T::default() })
    }
}

#[derive(Debug, Clone, Default)]
pub struct DbConnection<T> {
    pub conn: DatabaseConnection,
    #[allow(dead_code)]
    mark: T,
}

impl<T> From<DatabaseConnection> for DbConnection<T>
where
    T: Default,
{
    fn from(conn: DatabaseConnection) -> Self {
        Self { conn, mark: T::default() }
    }
}

impl<T> Deref for DbConnection<T> {
    type Target = DatabaseConnection;

    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

impl<T> AsRef<DatabaseConnection> for DbConnection<T> {
    fn as_ref(&self) -> &DatabaseConnection {
        &self.conn
    }
}
