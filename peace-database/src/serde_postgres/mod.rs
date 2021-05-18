//! # Serde Postgres
//!
//! Easily deserialize rows from postgres into
//! arbitrary structs. (Only deserialization is supported).
//!
//! This works with following postgres client libraries:
//!
//! - [`postgres`](//docs.rs/postgres)
//! - [`tokio_postgres`](//docs.rs/postgres/tokio_postgres)
//!
//! ```rust,no_run
//! use std::error::Error;
//! use serde::Deserialize;
//! use tokio_postgres::{connect, NoTls};
//!
//! #[derive(Clone, Debug, Deserialize)]
//! struct Person {
//!     name: String,
//!     age: i32,
//! }
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn Error>> {
//!     let (client, conn) = connect("postgres://postgres@localhost:5432", NoTls).await?;
//!     tokio::spawn(async move { conn.await.unwrap() });
//!
//!     client.execute(
//!         "CREATE TABLE IF NOT EXISTS Person (
//!             name VARCHAR NOT NULL,
//!             age INT NOT NULL
//!         )",
//!         &[]
//!     ).await?;
//!
//!     client.execute("INSERT INTO Person (name, age) VALUES ($1, $2)", &[&"Jane", &23]).await?;
//!
//!     client.execute("INSERT INTO Person (name, age) VALUES ($1, $2)", &[&"Alice", &32]).await?;
//!
//!     let rows = client.query("SELECT name, age FROM Person", &[]).await?;
//!
//!     let people: Vec<Person> = serde_postgres::from_rows(&rows)?;
//!
//!     for person in people {
//!         println!("{:?}", person);
//!     }
//!
//!     Ok(())
//! }
//! ```
#![deny(missing_docs)]

pub mod de;
mod raw;

pub use de::{from_row, from_rows, Deserializer, DeError, DeResult};
