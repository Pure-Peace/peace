#![allow(dead_code)]

use crate::settings::model::Settings;
use colored::Colorize;

use deadpool_redis::redis::{FromRedisValue, RedisError, RedisResult, ToRedisArgs};
use deadpool_redis::{cmd as _cmd, Cmd, Connection, Pool};
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
    ///
    /// # Try it:
    ///
    /// Will set expires to -1 (permanent)
    /// ```
    /// let _ = set("key", "value").await;
    /// ```
    /// Will set expires to 1000s
    /// ```
    /// let _ = set("key", &["value", "EX", "1000"]).await;
    /// ```
    /// "NX" means set if key not exists
    /// ```
    /// let _ = set("key", &["value", "EX", "1000", "NX"]).await;
    /// ```
    /// "XX" means set if key already exists
    /// ```
    /// let _ = set("key", &["value", "XX"]).await;
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
    /// let conn<deadpool_redis:Connection> = get_conn().await;
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
    /// let result = query("get", &["key"]).await;
    /// let result = query("get", "key").await;
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
    /// Will set expires to -1 (permanent)
    /// ```
    /// let _ = set("key", "value").await;
    /// ```
    /// Will set expires to 1000s
    /// ```
    /// let _ = set("key", &["value", "EX", "1000"]).await;
    /// ```
    /// "NX" means set if key not exists
    /// ```
    /// let _ = set("key", &["value", "EX", "1000", "NX"]).await;
    /// ```
    /// "XX" means set if key already exists
    /// ```
    /// let _ = set("key", &["value", "XX"]).await;
    /// ```
    pub async fn set<T: ToRedisArgs>(&self, key: &str, value: T) -> RedisResult<()> {
        let mut conn = self.get_conn().await;
        _cmd("SET")
            .arg(key)
            .arg(value)
            .execute_async(&mut conn)
            .await
    }

    /// Set a key, and return value before set
    ///
    /// # Examples:
    ///
    /// ```
    /// let _ = set("key", "value").await;
    /// let before = get_set("key", "value2").await; // will get "value" and set key to "value2"
    /// ```
    ///
    pub async fn get_set<T: ToRedisArgs, F: Send + FromRedisValue>(
        &self,
        key: &str,
        value: T,
    ) -> F {
        let mut conn = self.get_conn().await;
        _cmd("GETSET")
            .arg(key)
            .arg(value)
            .query_async(&mut conn)
            .await
            .unwrap()
    }

    /// Set multiple keys and values (MSET)
    ///
    /// # Examples:
    ///
    /// ```
    /// let _ = set_all(&[("key1", "value1"), ("key2", "value2")]).await;
    /// let results: Vec<String> = get_all(&["key1", "key2"]).await;
    /// println!("{:?}", results);
    /// ```
    ///
    pub async fn set_all<'a, K: ToRedisArgs, V: ToRedisArgs>(
        &self,
        items: &'a [(K, V)],
    ) -> RedisResult<()> {
        let mut conn = self.get_conn().await;
        _cmd("MSET").arg(items).execute_async(&mut conn).await
    }

    /// Get multiple keys (MGET)
    ///
    /// # Examples:
    ///
    /// ```
    /// let _ = set_all(&[("key1", "value1"), ("key2", "value2")]).await;
    /// let results: Vec<String> = get_all(&["key1", "key2"]).await;
    /// println!("{:?}", results);
    /// ```
    ///
    pub async fn get_all<T: ToRedisArgs, F: Send + FromRedisValue>(&self, args: T) -> F {
        let mut conn = self.get_conn().await;
        _cmd("MGET").arg(args).query_async(&mut conn).await.unwrap()
    }

    /// Delete a key or multiple keys
    /// Returns the number of deleted keys
    ///
    /// # Examples:
    /// ```
    /// let _ = set("key1", "value1").await; // set key1
    /// let count: u32 = del(&["key1", "key2"]).await; // will get 1, because key2 is not exists
    /// let count: u32 = del("key1").await; // delete one key
    /// ```
    ///
    pub async fn del<T: ToRedisArgs>(&self, keys: T) -> u32 {
        let mut conn = self.get_conn().await;
        _cmd("DEL").arg(keys).query_async(&mut conn).await.unwrap()
    }

    /// Set expire time to a key (seconds)
    ///
    /// # Examples:
    /// ```
    /// let _ = set("key", "value").await;
    /// let _ = expire("key", 0).await;
    /// let value: String = get("key").await;
    /// println!("{:?}", value);
    /// ```
    ///
    pub async fn expire<T: ToRedisArgs>(&self, key: T, expire: i32) -> RedisResult<()> {
        let mut conn = self.get_conn().await;
        _cmd("EXPIRE")
            .arg(key)
            .arg(expire)
            .execute_async(&mut conn)
            .await
    }

    /// Get single key from redis
    ///
    /// # Examples:
    ///
    /// ```
    /// let value: Result<String, _> = get("key").await;
    /// let value: String = get("key").await.unwrap();
    /// let value = get::<String>("key").await.unwrap();
    /// ```
    /// Default value
    /// ```
    /// let value = get("key").await.unwrap_or_else(|_| {
    ///     error!("failed to get key");
    ///     "default".to_string()
    /// });
    /// ```
    pub async fn get<T: Send + FromRedisValue, A: ToRedisArgs>(&self, key: A) -> Result<T, RedisError> {
        let mut conn = self.get_conn().await;
        match _cmd("GET").arg(key).query_async(&mut conn).await {
            Ok(res) => Ok(res),
            Err(e) => Err(e),
        }
    }
}
