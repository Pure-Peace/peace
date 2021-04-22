use base64::decode;
use bytes::Bytes;
use chrono::{DateTime, Local};
use derivative::Derivative;
use peace_constants::{GameMode, SubmissionStatus};
use peace_database::Database;
use pyo3::{types::PyBytes, PyErr, Python};
use std::time::Instant;
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;

use crate::objects::PlayMods;
use crate::utils::{self, MultipartData};

#[pg_mapper(table = "")]
#[derive(Debug, PostgresMapper)]
pub struct MiniScore {
    pub rank: i64,
    pub id: i64,
    pub accuracy: f32,
    pub pp_v2: Option<f32>,
    pub score: i32,
    pub combo: i32,
}

impl MiniScore {
    #[inline(always)]
    pub fn pp(&self) -> f32 {
        self.pp_v2.unwrap_or(0.0)
    }

    #[inline(always)]
    pub async fn from_database(
        player_id: i32,
        temp_table: &str,
        database: &Database,
    ) -> Option<Self> {
        match database.pg.query(
            &format!(r#"SELECT "rank", "id", accuracy, pp_v2, score, combo FROM {} WHERE user_id = $1 LIMIT 1"#, temp_table),
            &[&player_id],
        )
        .await
    {
        Ok(mut rows) => {
            if rows.len() > 0 {
                match MiniScore::from_row(rows.remove(0)) {
                    Ok(s) => Some(s),
                    Err(err) => {
                        error!(
                            "[MiniScore]: Failed to parse score, err: {:?}",
                            err
                        );
                        None
                    }
                }
            } else {
                None
            }
        },
        Err(err) => {
            error!(
                "[MiniScore]: Failed to get score, player {}, err: {:?};",
                player_id, err
            );
            None
        }
    }
    }
}

#[pg_mapper(table = "")]
#[derive(Debug, PostgresMapper)]
pub struct ScroeFromDatabase {
    pub rank: i64,
    pub id: i64,
    pub user_id: i32,
    pub name: String,
    pub u_name: Option<String>,
    pub pp_v2: Option<f32>,
    pub score: i32,
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
    pub fn to_string(&self, pp_as_score: bool, using_u_name: bool) -> String {
        format!("{id}|{name}|{score}|{combo}|{n50}|{n100}|{n300}|{miss}|{katu}|{geki}|{perfect}|{mods}|{user_id}|{rank}|{create_time}|{has_replay}",
            id = self.id,
            name = if using_u_name {
                self.u_name.as_ref().unwrap_or(&self.name)
            } else {
                &self.name
            },
            score = if pp_as_score { self.pp() as i32 } else {
                self.score
            },
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

    #[inline(always)]
    pub fn pp(&self) -> f32 {
        self.pp_v2.unwrap_or(0.0)
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct SubmitModular {
    pub quit: bool,     // x (quit 0 or 1)
    pub fail_time: i64, // ft (fail time)
    #[derivative(Debug = "ignore")]
    pub score: Vec<u8>, // score (base64 -> bytes)
    pub fs: String,
    pub beatmap_hash: String, // bmk
    pub c1: String,
    pub success_time: i64, // st (success time)
    pub password: String,  // pass (password)
    pub osu_version: i32,  // osuver
    // pub s: String, // s (s??) what's that?
    #[derivative(Debug = "ignore")]
    pub iv: Vec<u8>, // iv (initialization vector base64 -> bytes)
    #[derivative(Debug = "ignore")]
    pub score_file: Option<Bytes>, // score (replay file, octet-stream bytes lzma)
}

impl SubmitModular {
    #[inline(always)]
    pub fn from_mutipart(mut data: MultipartData) -> Option<Self> {
        Some(Self {
            quit: data.form::<i32>("x")? == 1,
            fail_time: data.form("ft")?,
            score: match decode(data.form::<String>("score")?) {
                Ok(s) => s,
                Err(_err) => return None,
            },
            fs: data.form("fs")?,
            beatmap_hash: data.form("bmk")?,
            c1: data.form("c1")?,
            success_time: data.form("st")?,
            password: data.form("pass")?,
            osu_version: data.form("osuver")?,
            // s: data.form("s")?, what
            iv: match decode(data.form::<String>("iv")?) {
                Ok(s) => s,
                Err(_err) => return None,
            },
            score_file: data.file("score"),
        })
    }

    #[inline(always)]
    /// Because Rust does not have an implementation of the rijndael algorithm,
    /// it is temporarily solved with the built-in python3 interpreter.
    pub fn python_decrypt(&self) -> Result<Vec<String>, PyErr> {
        debug!("[SubmitModular] Python decrypt start");
        let start = Instant::now();
        let gil = Python::acquire_gil();
        let python = gil.python();
        let module = python.import("__main__")?;

        let decryp_result = module
            .call_method1(
                "rijndael_cbc_decrypt",
                (
                    format!("osu!-scoreburgr---------{}", self.osu_version),
                    PyBytes::new(python, &self.iv),
                    PyBytes::new(python, &self.score),
                ),
            )?
            .extract()?;
        let end = start.elapsed();
        debug!(
            "[SubmitModular] Python decrypt success, time spent: {:?}",
            end
        );
        return Ok(decryp_result);
    }
}

#[derive(Debug)]
pub struct ScoreData {
    pub beatmap_md5: String,
    pub player_name: String,
    pub md5: String,
    pub n300: i32,
    pub n100: i32,
    pub n50: i32,
    pub geki: i32,
    pub katu: i32,
    pub miss: i32,
    pub score: i32,
    pub max_combo: i32,
    pub perfect: bool,
    pub grade: String,
    pub mods: PlayMods,
    pub status: SubmissionStatus,
    pub pass: bool,
    pub mode: GameMode,
    pub osu_version: i32,
    pub client_flags: i32,
    pub total_obj: Option<i32>,
    pub accuracy: Option<f32>,
}

impl ScoreData {
    #[inline(always)]
    pub async fn from_submit_modular(submit_data: &SubmitModular) -> Option<Self> {
        use utils::try_parse;
        let data = match submit_data.python_decrypt() {
            Ok(d) => d,
            Err(err) => {
                warn!("[SubmitModular] Python decrypt failed, err: {:?}", err);
                return None;
            }
        };
        // Check len
        if data.len() < 18 {
            warn!("[SubmitModular] Invalid score data length ( < 18)");
            return None;
        };
        let player_name = data[1].trim().to_string();
        // Check beatmap md5
        let beatmap_md5 = data[0].to_string();
        if beatmap_md5.len() != 32 {
            warn!(
                "[SubmitModular] Refused: {}; invalid scode_data beatmap hash({}).",
                player_name, beatmap_md5
            );
            return None;
        } else if beatmap_md5 != submit_data.beatmap_hash {
            warn!(
                "[SubmitModular] Refused: {}; decrypted score_data beatmap hash({}) not equal with submit_data({}).",
                player_name, beatmap_md5, submit_data.beatmap_hash
            );
            return None;
        };
        // Check score md5
        let md5 = data[2].to_string();
        if md5.len() != 32 {
            warn!(
                "[SubmitModular] Refused: {}; invalid score md5({}).",
                player_name, md5
            );
            return None;
        };
        // Check osu version
        let osu_version = try_parse::<i32>(&data[17][..8])?;
        if osu_version != submit_data.osu_version {
            warn!(
                "[SubmitModular] Refused: {}; decrypted osu version({}) not equal({}).",
                player_name, osu_version, submit_data.osu_version
            );
            return None;
        }
        let client_flags = {
            let mut count = 0;
            for i in data[17].chars() {
                if i == ' ' {
                    count += 1;
                }
            }
            count
        };
        let mods = PlayMods::parse(try_parse(&data[13])?);
        let mode = GameMode::parse_with_playmod(try_parse(&data[15])?, &mods.list)?;

        let pass = &data[14] == "True";
        let mut data = Self {
            beatmap_md5,
            player_name,
            md5,
            n300: try_parse(&data[3])?,
            n100: try_parse(&data[4])?,
            n50: try_parse(&data[5])?,
            geki: try_parse(&data[6])?,
            katu: try_parse(&data[7])?,
            miss: try_parse(&data[8])?,
            score: try_parse(&data[9])?,
            max_combo: try_parse(&data[10])?,
            perfect: &data[11] == "True",
            grade: if pass {
                data[12].to_string()
            } else {
                "F".to_string()
            },
            mods,
            status: if pass {
                SubmissionStatus::Passed
            } else {
                SubmissionStatus::Failed
            },
            pass,
            mode,
            osu_version,
            client_flags,
            total_obj: None,
            accuracy: None,
        };

        data.total_obj = Some(data.total_obj_count(false));
        data.accuracy = Some(data.calc_acc(false));

        Some(data)
    }

    #[inline(always)]
    pub fn query(&self) -> String {
        let mut query = format!(
            "md5={}&mode={}&mods={}&n300={}&n100={}&n50={}&katu={}&combo={}&miss={}",
            self.beatmap_md5,
            self.mode.raw_value(),
            self.mods.value,
            self.n300,
            self.n100,
            self.n50,
            self.katu,
            self.max_combo,
            self.miss
        );
        if !self.pass {
            query += &format!("&passed_obj={}", self.total_obj_count(false));
        };
        query
    }

    #[inline(always)]
    pub fn total_obj_count(&self, recalc: bool) -> i32 {
        if self.total_obj.is_some() && !recalc {
            return self.total_obj.unwrap();
        };
        let base = self.n300 + self.n50 + self.n100 + self.miss;
        match self.mode.raw_value() {
            0 => base,
            1 => base - self.n50,
            2 => base + self.katu,
            3 => base + self.geki + self.katu,
            x => {
                error!("[ScoreData] what happened? why {:?}={}?", self, x);
                base
            }
        }
    }

    #[inline(always)]
    pub fn calc_acc(&self, recalc: bool) -> f32 {
        if self.accuracy.is_some() && !recalc {
            return self.accuracy.unwrap();
        };
        let total = self.total_obj_count(recalc);
        if total == 0 {
            return 0.0;
        };
        let val = match self.mode.raw_value() {
            0 => {
                (self.n300 * 300 + self.n100 * 100 + self.n50 * 50) as f32 / ((total * 300) as f32)
            }
            1 => (self.n300 as f32 + self.n100 as f32 * 0.5) / (total as f32),
            2 => (self.n300 + self.n100 + self.n50) as f32 / (total as f32),
            3 => {
                ((self.n300 + self.geki) * 300 + self.katu * 200 + self.n100 * 100 + self.n50 * 50)
                    as f32
                    / ((total * 300) as f32)
            }
            x => {
                error!("[ScoreData] what happened? why {:?}={}?", self, x);
                0f32
            }
        } * 100.0;
        val
    }
}
