use crate::set_with_db;
use crate::{database::Database, utils};
use chrono::{DateTime, Utc};
use tokio_pg_mapper_derive::PostgresMapper;

set_with_db! {
    table="user";
    schema="settings";
    #[pg_mapper(table = "user.settings")]
    #[derive(Clone, Debug, PostgresMapper)]
    pub struct PlayerSettings {
        pub id: i32,
        pub game_mode: i16,
        pub language: String,
        pub in_game_translate: bool,
        pub pp_scoreboard: bool,
        pub update_time: DateTime<Utc>,
    }
}

impl PlayerSettings {
    #[inline(always)]
    /// Initial PlayerSettings from database
    pub async fn from_database(user_id: i32, database: &Database) -> Option<PlayerSettings> {
        utils::struct_from_database("user", "settings", "id", "*", &user_id, database).await
    }

    #[inline(always)]
    /// Update PlayerSettings from database
    pub async fn update(&mut self, database: &Database) -> bool {
        let start = std::time::Instant::now();
        let new = PlayerSettings::from_database(self.id, database).await;
        if new.is_none() {
            error!("PlayerSettings update failed.");
            return false;
        };
        *self = new.unwrap();
        info!(
            "New PlayerSettings ({}) updated in {:?}; update time: {}",
            self.id,
            start.elapsed(),
            self.update_time
        );
        true
    }
}
