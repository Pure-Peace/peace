use super::depends::*;
#[derive(Debug, Clone, Deserialize)]
pub struct Stats {
    #[serde(skip_deserializing)]
    #[serde(default = "default_rank")]
    pub rank: i32,
    pub pp_v1: f32,
    pub pp_v2: f32,
    pub accuracy: f32,
    pub total_hits: i32,
    pub total_score: i64,
    pub ranked_score: i64,
    pub playcount: i32,
    pub playtime: i64,
    pub max_combo: i32,
    #[serde(skip_deserializing)]
    #[serde(default = "default_time")]
    pub update_time: DateTime<Local>,
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            rank: 100000,
            pp_v1: 0.0,
            pp_v2: 0.0,
            accuracy: 0.0,
            total_hits: 0,
            total_score: 0,
            ranked_score: 0,
            playcount: 0,
            playtime: 0,
            max_combo: 0,
            update_time: Local::now(),
        }
    }

    #[inline(always)]
    pub fn update_time(&mut self) {
        self.update_time = Local::now()
    }

    #[inline(always)]
    pub async fn calc_rank_from_database(&mut self, mode: &GameMode, database: &Database) {
        // TODO: Support for ppv1!!!!! current default is ppv2!~!!!
        let recalculate_start = Instant::now();
        let default_performance = format!("pp_v2{}", mode.sub_mod_table());
        let sql = format!(
            r#"SELECT "rank"::INT4 FROM (
                SELECT 
                    "id", "{0}", RANK() OVER (ORDER BY "{0}" DESC) 
                FROM 
                    "game_stats"."{1}" 
                LEFT JOIN 
                    "user"."base" USING("id") 
                WHERE ("privileges" & 1 > 0)) AS "R"
            WHERE "R"."{0}" = $1 LIMIT 1;"#,
            default_performance,
            mode.mode_name()
        );

        match database.pg.query_first(&sql, &[&self.pp_v2]).await {
            Ok(row) => self.rank = row.get("rank"),
            Err(err) => {
                error!("Failed to recalculate player's rank: {:?}", err);
                return;
            }
        };

        self.update_time();
        let recalculate_end = recalculate_start.elapsed();
        debug!("Rank calculate done, time spent: {:.2?}", recalculate_end);
    }
}

#[inline(always)]
fn default_rank() -> i32 {
    100000
}

#[inline(always)]
fn default_time() -> DateTime<Local> {
    Local::now()
}
