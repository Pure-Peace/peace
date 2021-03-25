use crate::database::Database;
use crate::set_with_db;
use chrono::{DateTime, Utc};
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;

set_with_db! {
    table="user";
    schema="info";
    #[pg_mapper(table = "user.info")]
    #[derive(Clone, Debug, PostgresMapper)]
    pub struct PlayerInfo {
        pub id: i32,
        pub is_bot: bool,
        pub cheat: bool,
        pub multiaccount: bool,
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
        let row = database
            .pg
            .query_first(
                r#"SELECT * FROM "user"."info" WHERE "id" = $1;"#,
                &[&user_id],
            )
            .await;
        if row.is_err() {
            error!(
                "Failed to get user info{} from database! error: {:?}",
                user_id, row
            );
            return None;
        }

        let row = row.unwrap();
        let result = PlayerInfo::from_row(row);
        if result.is_err() {
            error!(
                "Failed to deserialize PlayerInfo from pg-row! error: {:?}",
                result
            );
            return None;
        };

        Some(result.unwrap())
    }

    /*     pub async fn set_with_db(&mut self, field: &str, value: , database: &Database) -> Result<()> {

       }
    */
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
