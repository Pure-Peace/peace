#![allow(dead_code)]

use crate::settings::model::Settings;

use colored::Colorize;
use deadpool_postgres::{Client, Pool};
use tokio_postgres::{types::ToSql, Error, NoTls, Row, Statement};

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
        // Create pool
        print!(
            "> {}",
            "Creating postgres connection pool...".bright_purple()
        );
        let pool = settings.postgres.create_pool(NoTls).unwrap();
        let pool_status = format!("Max size: {}", pool.status().max_size).green();
        println!(" {} -> {}", "OK".green(), pool_status);
        // Check connection, it will panic if failed
        if settings.check_pools_on_created == true {
            print!("> {}", "Check postgres connection...".bright_purple());
            pool.get()
                .await
                .expect("Please make sure you can connect to the postgres.");
            println!(" {}", "OK".green());
        };
        Postgres { pool }
    }

    #[inline(always)]
    /// Test whether the connection pool can connect to the postgres
    ///
    /// Will returns bool
    pub async fn is_connected(&self) -> bool {
        match self.pool.get().await {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    #[inline(always)]
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

    #[inline(always)]
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

    #[inline(always)]
    pub async fn batch_execute(&self, query: &str) -> Result<(), Error> {
        let c = self.get_client().await;
        c.batch_execute(query).await
    }

    #[inline(always)]
    /// Query and get all result rows
    ///
    /// # Examples:
    ///
    /// ```
    /// let rows: Vec<Row> = query("YOUR SQL $1", ["params $1-$n", ...]);
    /// ```
    pub async fn query(
        &self,
        query: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<Row>, Error> {
        let (c, s) = self.get_ready(query).await;
        c.query(&s, params).await
    }

    #[inline(always)]
    /// Query and query the frist result row
    /// ### The query using this method must return a result row.
    /// ### If you are not sure whether the result will be returned, please use get_all instead of get_first.
    ///
    /// # Examples:
    ///
    /// ```
    /// let row: Row = query_first("YOUR SQL $1", ["params $1-$n", ...]);
    /// ```
    pub async fn query_first(
        &self,
        query: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Row, Error> {
        let (c, s) = self.get_ready(query).await;
        c.query_one(&s, params).await
    }

    #[inline(always)]
    /// Query and query the frist result row (no params)
    ///
    /// # Examples:
    ///
    /// ```
    /// let row: Row = query_first_simple("YOUR SQL");
    /// ```
    pub async fn query_first_simple(&self, query: &str) -> Result<Row, Error> {
        self.query_first(query, &[]).await
    }

    #[inline(always)]
    /// Query and query all result rows (no params)
    ///
    /// # Examples:
    ///
    /// ```
    /// let row: Vec<Row> = query_simple("YOUR SQL");
    /// ```
    pub async fn query_simple(&self, query: &str) -> Result<Vec<Row>, Error> {
        self.query(query, &[]).await
    }

    #[inline(always)]
    /// Query and return the number of result rows
    ///
    /// # Examples:
    ///
    /// ```
    /// let size: u64 = execute("YOUR SQL $1", ["params $1-$n", ...]);
    /// ```
    pub async fn execute(
        &self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + std::marker::Sync)],
    ) -> Result<u64, Error> {
        let (c, s) = self.get_ready(query).await;
        c.execute(&s, params).await
    }

    #[inline(always)]
    /// Query and return the number of result rows (no params)
    ///
    /// # Examples:
    ///
    /// ```
    /// let size: u64 = execute_simple("YOUR SQL");
    /// ```
    pub async fn execute_simple(&self, query: &str) -> Result<u64, Error> {
        self.execute(query, &[]).await
    }
}
