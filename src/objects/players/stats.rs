use super::depends::*;
#[derive(Debug, Clone)]
pub struct Stats {
    pub rank: i32,
    pub performance_v1: i16,
    pub performance_v2: i16,
    pub accuracy: f32,
    pub total_score: i64,
    pub ranked_score: i64,
    pub playcount: i32,
    pub playtime: i64,
    pub max_combo: i32,
    pub update_time: DateTime<Local>,
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            rank: 100000,
            performance_v1: 0,
            performance_v2: 0,
            accuracy: 0.0,
            total_score: 0,
            ranked_score: 0,
            playcount: 0,
            playtime: 0,
            max_combo: 0,
            update_time: Local::now(),
        }
    }
}