#![allow(dead_code)]

use crate::settings::types::Settings;

use deadpool_postgres::{Client, Pool};
use tokio_postgres::{types::ToSql, NoTls, Row, Statement};


/// Postgres object
#[derive(Clone)]
pub struct Postgres {
    pub pool: Pool,
}

impl Postgres {
    /// Create postgres objects
    ///
    /// The postgres object including a dead connection pool
    ///
    /// # Examples:
    ///
    /// ```
    /// use crate::settings::Settings;
    ///
    /// let settings = Settings::new();
    /// let postgres = Postgres::new(settings: Settings);
    /// ```
    /// if check_pools_on_created is true, will test usability when creating connection pool
    pub async fn new(settings: &Settings) -> Self {
        let pool = settings.postgres.create_pool(NoTls).unwrap();
        if settings.check_pools_on_created == true {
            pool.get()
                .await
                .expect("Please make sure you can connect to the postgres.");
        };
        Postgres { pool }
    }

    /// Test whether the connection pool can connect to the postgres
    ///
    /// Will returns bool
    pub async fn is_connected(&self) -> bool {
        match self.pool.get().await {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Ready to query
    ///
    /// # Examples:
    ///
    /// ```
    /// let (client, statment) = get_ready("YOUR SQL");
    /// let result = client.query(statment, ["params $1-$n", ...]);
    /// ```
    pub async fn get_ready(&self, query: &str) -> (Client, Statement) {
        let client = self.get_client().await;
        let statement = client.prepare(query).await.unwrap();
        (client, statement)
    }

    /// Get postgres client from deadpool
    ///
    /// # Examples:
    ///
    /// ```
    /// let client<deadpool_postgres:Client> = get_client();
    /// ```
    pub async fn get_client(&self) -> Client {
        self.pool
            .get()
            .await
            .expect("Unable to get postgres client!")
    }

    /// Query and get all result rows
    ///
    /// # Examples:
    ///
    /// ```
    /// let rows: Vec<Row> = get_all("YOUR SQL $1", ["params $1-$n", ...]);
    /// ```
    pub async fn get_all(&self, query: &str, params: &[&(dyn ToSql + Sync)]) -> Vec<Row> {
        let (c, s) = self.get_ready(query).await;
        let rows = c.query(&s, params).await.unwrap();
        rows
    }

    /// Query and get the frist result row
    /// ### The query using this method must return a result row.
    /// ### If you are not sure whether the result will be returned, please use get_all instead of get_first.
    ///
    /// # Examples:
    ///
    /// ```
    /// let row: Row = get_first("YOUR SQL $1", ["params $1-$n", ...]);
    /// ```
    pub async fn get_first(&self, query: &str, params: &[&(dyn ToSql + Sync)]) -> Row {
        let (c, s) = self.get_ready(query).await;
        let row = c
            .query_one(&s, params)
            .await
            .expect("Maybe this query did not return result rows?
            But the query using this method must return a result row.
            If you are not sure whether the result will be returned, please use get_all instead of get_first.");
        row
    }

    /// Query and get the frist result row (no params)
    ///
    /// # Examples:
    ///
    /// ```
    /// let row: Row = get_first_simple("YOUR SQL");
    /// ```
    pub async fn get_first_simple(&self, query: &str) -> Row {
        self.get_first(query, &[]).await
    }

    /// Query and get all result rows (no params)
    ///
    /// # Examples:
    ///
    /// ```
    /// let row: Vec<Row> = get_all_simple("YOUR SQL");
    /// ```
    pub async fn get_all_simple(&self, query: &str) -> Vec<Row> {
        self.get_all(query, &[]).await
    }

    /// Query and return the number of result rows
    ///
    /// # Examples:
    ///
    /// ```
    /// let size: u64 = execute("YOUR SQL $1", ["params $1-$n", ...]);
    /// ```
    pub async fn execute(&self, query: &str, params: &[&(dyn tokio_postgres::types::ToSql + std::marker::Sync)]) -> u64 {
        let (c, s) = self.get_ready(query).await;
        let size = c.execute(&s, params).await.unwrap();
        size
    }

    /// Query and return the number of result rows (no params)
    ///
    /// # Examples:
    ///
    /// ```
    /// let size: u64 = execute_simple("YOUR SQL");
    /// ```
    pub async fn execute_simple(&self, query: &str) -> u64 {
        self.execute(query, &[]).await
    }
}
