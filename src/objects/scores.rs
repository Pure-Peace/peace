use chrono::{DateTime, Local};
use tokio_pg_mapper_derive::PostgresMapper;

#[pg_mapper(table = "")]
#[derive(Debug, PostgresMapper)]
pub struct ScroeFromDatabase {
    pub rank: i64,
    pub id: i64,
    pub user_id: i32,
    pub name: String,
    pub any_score: i32,
    pub combo: i32,
    pub n50: i32,
    pub n100: i32,
    pub n300: i32,
    pub miss: i32,
    pub katu: i32,
    pub geki: i32,
    pub perfect: bool,
    pub mods: i32,
    pub create_time: DateTime<Local>,
}

impl ScroeFromDatabase {
    #[inline(always)]
    pub fn to_string(&self) -> String {
        format!("{id}|{name}|{score}|{combo}|{n50}|{n100}|{n300}|{miss}|{katu}|{geki}|{perfect}|{mods}|{user_id}|{rank}|{create_time}|{has_replay}",
            id = self.id,
            name = self.name,
            score = self.any_score,
            combo = self.combo,
            n50 = self.n50,
            n100 = self.n100,
            n300 = self.n300,
            miss = self.miss,
            katu = self.katu,
            geki = self.geki,
            perfect = if self.perfect { "1" } else { "0" },
            mods = self.mods,
            user_id = self.user_id,
            rank = self.rank,
            create_time = self.create_time.timestamp(),
            has_replay = "1"
        )
    }
}
