use crate::set_with_db;
use crate::{database::Database, utils};
use chrono::{DateTime, Utc};
use tokio_pg_mapper_derive::PostgresMapper;

set_with_db! {
    table="user";
    schema="info";
    #[pg_mapper(table = "user.info")]
    #[derive(Clone, Debug, PostgresMapper)]
    pub struct PlayerInfo {
        pub id: i32,
        pub credit: i32,
        pub is_bot: bool,
        pub cheat: i32,
        pub multiaccount: i32,
        pub donor_start: Option<DateTime<Utc>>,
        pub silence_start: Option<DateTime<Utc>>,
        pub restrict_start: Option<DateTime<Utc>>,
        pub ban_start: Option<DateTime<Utc>>,
        pub donor_end: Option<DateTime<Utc>>,
        pub silence_end: Option<DateTime<Utc>>,
        pub restrict_end: Option<DateTime<Utc>>,
        pub ban_end: Option<DateTime<Utc>>,
        pub last_login_time: Option<DateTime<Utc>>,
        pub discord_verifyed_time: Option<DateTime<Utc>>,
        pub qq_verifyed_time: Option<DateTime<Utc>>,
        pub official_verifyed_time: Option<DateTime<Utc>>,
        pub osu_verifyed_time: Option<DateTime<Utc>>,
        pub mail_verifyed_time: Option<DateTime<Utc>>,
        pub update_time: DateTime<Utc>,
    }
}

impl PlayerInfo {
    #[inline(always)]
    /// Initial pleyer info from database
    pub async fn from_database(user_id: i32, database: &Database) -> Option<PlayerInfo> {
        utils::struct_from_database("user", "info", "id", "*", &user_id, database).await
    }

    #[inline(always)]
    /// Update player info from database
    pub async fn update(&mut self, database: &Database) -> bool {
        let start = std::time::Instant::now();
        let new = PlayerInfo::from_database(self.id, database).await;
        if new.is_none() {
            error!("PlayerInfo update failed.");
            return false;
        };
        *self = new.unwrap();
        info!(
            "New PlayerInfo ({}) updated in {:?}; update time: {}",
            self.id,
            start.elapsed(),
            self.update_time
        );
        true
    }
}
