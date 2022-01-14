#![allow(dead_code)]

use {
    colored::Colorize,
    deadpool_postgres::{Client, Pool, PoolError},
    std::fmt,
    tokio_pg_mapper::FromTokioPostgresRow,
    tokio_postgres::{types::ToSql, Error, NoTls, Row, Statement},
};

pub enum PostgresError {
    PoolError,
    DbError(Error),
}

impl PostgresError {
    #[inline(always)]
    pub fn from_pg_err(pg_error: Error) -> Self {
        Self::DbError(pg_error)
    }
}

impl fmt::Debug for PostgresError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PoolError => fmt.write_str("PoolError"),
            Self::DbError(err) => fmt.write_fmt(format_args!("{:?}", err)),
        }
    }
}

impl fmt::Display for PostgresError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PoolError => fmt.write_str("PoolError"),
            Self::DbError(err) => fmt.write_fmt(format_args!("{}", err)),
        }
    }
}

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
    /// if check_connect is true, will test usability when creating connection pool
    pub async fn new(config: &deadpool_postgres::Config, check_connect: bool) -> Self {
        // Create pool
        print!(
            "> {}",
            "Creating postgres connection pool...".bright_purple()
        );
        let pool = config.create_pool(NoTls).unwrap();
        let pool_status = format!("Max size: {}", pool.status().max_size).green();
        println!(" {} -> {}", "OK".green(), pool_status);
        // Check connection, it will panic if failed
        if check_connect {
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
    /// ```rust,ignore
    /// let (client, statment) = get_ready("YOUR SQL");
    /// let result = client.query(statment, ["params $1-$n", ...]);
    /// ```
    pub async fn get_ready(&self, query: &str) -> Result<(Client, Statement), PostgresError> {
        let client = self.get_client().await?;
        let statement = client
            .prepare(query)
            .await
            .map_err(PostgresError::from_pg_err)?;
        Ok((client, statement))
    }

    #[inline(always)]
    /// Get postgres client from deadpool
    ///
    /// # Examples:
    ///
    /// ```rust,ignore
    /// let client<deadpool_postgres:Client> = get_client();
    /// ```
    pub async fn get_client(&self) -> Result<Client, PostgresError> {
        self.pool.get().await.map_err(|e| match e {
            PoolError::Backend(e) => PostgresError::from_pg_err(e),
            _ => PostgresError::PoolError,
        })
    }

    #[inline(always)]
    pub async fn batch_execute(&self, query: &str) -> Result<(), PostgresError> {
        let c = self.get_client().await?;
        c.batch_execute(query)
            .await
            .map_err(PostgresError::from_pg_err)
    }

    #[inline(always)]
    /// Query and get all result rows
    ///
    /// # Examples:
    ///
    /// ```rust,ignore
    /// let rows: Vec<Row> = query("YOUR SQL $1", ["params $1-$n", ...]);
    /// ```
    pub async fn query(
        &self,
        query: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<Row>, PostgresError> {
        let (c, s) = self.get_ready(query).await?;
        c.query(&s, params)
            .await
            .map_err(PostgresError::from_pg_err)
    }

    #[inline(always)]
    /// Query and query the frist result row
    /// ### The query using this method must return a result row.
    /// ### If you are not sure whether the result will be returned, please use get_all instead of get_first.
    ///
    /// # Examples:
    ///
    /// ```rust,ignore
    /// let row: Row = query_first("YOUR SQL $1", ["params $1-$n", ...]);
    /// ```
    pub async fn query_first(
        &self,
        query: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Row, PostgresError> {
        let (c, s) = self.get_ready(query).await?;
        c.query_one(&s, params)
            .await
            .map_err(PostgresError::from_pg_err)
    }

    #[inline(always)]
    /// Query and query the frist result row (no params)
    ///
    /// # Examples:
    ///
    /// ```rust,ignore
    /// let row: Row = query_first_simple("YOUR SQL");
    /// ```
    pub async fn query_first_simple(&self, query: &str) -> Result<Row, PostgresError> {
        self.query_first(query, &[]).await
    }

    #[inline(always)]
    /// Query and query all result rows (no params)
    ///
    /// # Examples:
    ///
    /// ```rust,ignore
    /// let row: Vec<Row> = query_simple("YOUR SQL");
    /// ```
    pub async fn query_simple(&self, query: &str) -> Result<Vec<Row>, PostgresError> {
        self.query(query, &[]).await
    }

    #[inline(always)]
    /// Query and return the number of result rows
    ///
    /// # Examples:
    ///
    /// ```rust,ignore
    /// let size: u64 = execute("YOUR SQL $1", ["params $1-$n", ...]);
    /// ```
    pub async fn execute(
        &self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + std::marker::Sync)],
    ) -> Result<u64, PostgresError> {
        let (c, s) = self.get_ready(query).await?;
        c.execute(&s, params)
            .await
            .map_err(PostgresError::from_pg_err)
    }

    #[inline(always)]
    /// Query and return the number of result rows (no params)
    ///
    /// # Examples:
    ///
    /// ```rust,ignore
    /// let size: u64 = execute_simple("YOUR SQL");
    /// ```
    pub async fn execute_simple(&self, query: &str) -> Result<u64, PostgresError> {
        self.execute(query, &[]).await
    }

    #[inline(always)]
    pub async fn struct_from_database<T: FromTokioPostgresRow>(
        &self,
        query: &str,
        param: &[&(dyn ToSql + Sync)],
    ) -> Option<T> {
        let row = self.query_first(query, param).await;
        if let Err(err) = row {
            debug!(
                "[struct_from_database] Failed to get row from database, error: {:?}",
                err
            );
            return None;
        }

        let row = row.unwrap();
        match <T>::from_row(row) {
            Ok(result) => Some(result),
            Err(err) => {
                let type_name = std::any::type_name::<T>();
                error!(
                    "[struct_from_database] Failed to deserialize {} from pg-row! error: {:?}",
                    type_name, err
                );
                None
            }
        }
    }

    #[inline(always)]
    pub async fn structs_from_database<T: FromTokioPostgresRow>(
        &self,
        query: &str,
        param: &[&(dyn ToSql + Sync)],
    ) -> Option<Vec<T>> {
        let rows = self.query(query, param).await;
        if let Err(err) = rows {
            debug!(
                "[struct_from_database] Failed to get row from database, error: {:?}",
                err
            );
            return None;
        }

        let rows = rows.unwrap();
        let mut structs = Vec::with_capacity(rows.len());
        for row in rows {
            match <T>::from_row(row) {
                Ok(t) => structs.push(t),
                Err(err) => {
                    let type_name = std::any::type_name::<T>();
                    error!(
                        "[struct_from_database] Failed to deserialize {} from pg-row! error: {:?}",
                        type_name, err
                    );
                }
            };
        }
        Some(structs)
    }

    #[inline(always)]
    pub async fn struct_from_database_simple<T: FromTokioPostgresRow>(
        &self,
        table: &str,
        schema: &str,
        query_by: &str,
        fields: &str,
        param: &(dyn tokio_postgres::types::ToSql + Sync),
    ) -> Option<T> {
        let query = format!(
            "SELECT {} FROM \"{}\".\"{}\" WHERE \"{}\" = $1;",
            fields, table, schema, query_by
        );
        debug!("[struct_from_database] query: {}", query);
        self.struct_from_database(&query, &[param]).await
    }
}

#[macro_export]
macro_rules! set_with_db {
    (
        table=$table:expr;
        schema=$schema:expr;
        $(#[$meta:meta])*
        $vis:vis struct $struct_name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field_name:ident : $field_type:ty
            ),*$(,)+
        }
    ) => {
        $(#[$meta])*
        $vis struct $struct_name{
            $(
                $(#[$field_meta:meta])*
                $field_vis $field_name: $field_type,
            )*
        }
        paste::paste! {
            impl $struct_name {
                $(
                    pub async fn [<set_ $field_name _with_db>](&mut self, value: $field_type, database: &Database) -> bool {
                        let query = concat!(r#"UPDATE "#, stringify!($table), r#"."#, stringify!($schema), r#" SET ""#, stringify!($field_name), r#"" = $1 WHERE "id" = $2"#);
                        let res = match database.pg.execute(query, &[&value, &self.id]).await {
                            Ok(_) => true,
                            Err(err) => {
                                warn!(
                                    stringify!("[set_with_db] Failed to set "$struct_name"."$field_name" to table "$table", err: ""{:?}"),
                                    err
                                );
                                false
                            }
                        };
                        self.$field_name = value;
                        res
                    }
                )*
            }
        }
    }
}
