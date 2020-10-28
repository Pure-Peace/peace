#![allow(dead_code)]

use crate::settings::types::Settings;
use colored::Colorize;

use deadpool_redis::{
    cmd as _cmd,
    redis::{FromRedisValue, RedisResult, ToRedisArgs},
    Cmd, Connection, Pool,
};
use std::marker::Send;

/// Redis object
///
/// # Examples:
///
/// ```
/// // normal
/// let mut conn = pool.get().await.unwrap();
/// let res: String = cmd("get").arg("a").query_async(&mut conn).await.unwrap();
///
/// // extend
/// let cmd = cmd(name: "get").arg("e");
/// let res: i32 = query_cmd(cmd).await;
/// // get and set
/// let res = set(key: "gg", value: 1).await;
/// let res: i32 = get(key: "gg").await;
/// // more
/// let res: i32 = query(name: "get", arg: "gg").await;
/// let res: String = query(name: "get", arg: &["gg"]).await;
/// let z = execute(name: "set", arg: &["ok", "no"]).await;
///
/// println!("{:?}", res);
/// ```
#[derive(Clone)]
pub struct Redis {
    pub pool: Pool,
}

impl Redis {
    /// Create redis objects
    ///
    /// The redis object including a dead connection pool
    ///
    /// # Examples:
    ///
    /// ```
    /// use crate::settings::Settings;
    ///
    /// let settings = Settings::new();
    /// let redis = Redis::new(settings: Settings);
    /// ```
    /// if check_pools_on_created is true, will test usability when creating connection pool
    pub async fn new(settings: &Settings) -> Self {
        // Create pool
        print!("> {}", "Creating redis connection pool...".bright_purple());
        let pool = settings.redis.create_pool().unwrap();
        let pool_status = format!("Max size: {}", pool.status().max_size).green();
        println!(" {} -> {}", "OK".green(), pool_status);
        // Check connection, it will panic if failed
        if settings.check_pools_on_created == true {
            print!("> {}", "Check redis connection...".bright_purple());
            pool.get()
                .await
                .expect("Please make sure you can connect to the redis.");
            println!(" {}", "OK".green());
        };
        Redis { pool }
    }

    /// Test whether the connection pool can connect to the redis
    ///
    /// Will returns bool
    pub async fn is_connected(&self) -> bool {
        match self.pool.get().await {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Get redis conn from deadpool
    ///
    /// # Examples:
    ///
    /// ```
    /// let conn<deadpool_redis:Connection> = get_conn();
    /// ```
    pub async fn get_conn(&self) -> Connection {
        self.pool
            .get()
            .await
            .expect("Unable to get redis connection!")
    }

    /// Redis command
    ///
    /// https://docs.rs/deadpool-redis/0.6.1/deadpool_redis/struct.Cmd.html
    ///
    /// # Examples:
    ///
    /// ```
    /// let mut conn = get_conn().await.unwrap();
    ///
    /// // query
    /// let get_result = cmd("get").arg("key").query_async(&mut conn).await.unwrap();
    /// // execute
    /// let _ = cmd("set").arg(&["key", "value"]).execute_async(&mut conn).await.unwrap();
    /// ```
    pub fn cmd(&self, name: &str) -> Cmd {
        _cmd(name)
    }

    /// Query a raw cmd
    ///
    /// # Example:
    ///
    /// ```
    /// let result = query("get", &["key"])
    /// let result = query("get", "key")
    /// ```
    pub async fn query<T: Send + FromRedisValue, ARG: ToRedisArgs>(
        &self,
        name: &str,
        arg: ARG,
    ) -> T {
        let mut conn = self.get_conn().await;
        _cmd(name).arg(arg).query_async(&mut conn).await.unwrap()
    }

    /// Execute a raw cmd
    ///
    /// # Example:
    ///
    /// ```
    /// let _ = execute("set", &["key", "value"])
    /// ```
    pub async fn execute<T: ToRedisArgs>(&self, name: &str, arg: T) -> RedisResult<()> {
        let mut conn = self.get_conn().await;
        _cmd(name).arg(arg).execute_async(&mut conn).await
    }

    /// Query a Cmd object
    ///
    /// # Examples:
    ///
    /// ```
    /// let cmd_object = cmd("set").arg(&["key", "value"]);
    /// let _ = query_cmd(cmd_object).await;
    /// ```
    pub async fn query_cmd<T: Send + FromRedisValue>(&self, cmd: &mut Cmd) -> T {
        let mut conn = self.get_conn().await;
        cmd.query_async(&mut conn).await.unwrap()
    }

    /// Execute a Cmd object
    ///
    /// # Examples:
    ///
    /// ```
    /// let cmd_object = cmd("set").arg(&["key", "value"]);
    /// let _ = execute_cmd(cmd_object).await;
    /// ```
    pub async fn execute_cmd(&self, cmd: &Cmd) -> RedisResult<()> {
        let mut conn = self.get_conn().await;
        cmd.execute_async(&mut conn).await
    }

    /// Set a key to redis
    ///
    /// # Examples:
    ///
    /// ```
    /// let _ = set("key", "value");
    /// ```
    pub async fn set<T: ToRedisArgs>(&self, key: &str, value: T) -> RedisResult<()> {
        let mut conn = self.get_conn().await;
        _cmd("SET")
            .arg(key)
            .arg(value)
            .execute_async(&mut conn)
            .await
    }

    /// Get a key from redis
    ///
    /// # Examples:
    ///
    /// ```
    /// let value: String = get("key");
    /// let value: i32 = get("key");
    /// ```
    pub async fn get<T: Send + FromRedisValue>(&self, key: &str) -> T {
        let mut conn = self.get_conn().await;
        _cmd("GET").arg(key).query_async(&mut conn).await.unwrap()
    }
}
