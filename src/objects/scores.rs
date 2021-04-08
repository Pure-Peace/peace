use base64::decode;
use bytes::Bytes;
use chrono::{DateTime, Local};
use pyo3::{types::PyBytes, PyErr, Python};
use std::time::Instant;
use tokio_pg_mapper_derive::PostgresMapper;

use crate::utils::{self, MultipartData};
use crate::{constants::GameMode, objects::PlayMods};

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

#[derive(Debug)]
pub struct SubmitModular {
    pub quit: bool,     // x (quit 0 or 1)
    pub fail_time: i32, // ft (fail time)
    pub score: Vec<u8>, // score (base64 -> bytes)
    pub fs: String,
    pub beatmap_hash: String, // bmk
    pub c1: String,
    pub st: i32,
    pub password: String,    // pass (password)
    pub osu_version: i32,    // osuver
    pub client_hash: String, // s (client_hash)
    pub iv: Vec<u8>,         // iv (initialization vector base64 - bytes)

    pub score_data: Bytes, // score (replay file, octet-stream bytes)
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
            st: data.form("st")?,
            password: data.form("pass")?,
            osu_version: data.form("osuver")?,
            client_hash: data.form("s")?,
            iv: match decode(data.form::<String>("iv")?) {
                Ok(s) => s,
                Err(_err) => return None,
            },
            score_data: data.file("score")?,
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
    pub score_md5: String,
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
    pub pass: bool,
    pub mode: GameMode,
    pub osu_version: i32,
    pub client_flags: i32,
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
        if beatmap_md5.len() != 32 || beatmap_md5 != submit_data.beatmap_hash {
            warn!(
                "[SubmitModular] Refused: {}; decrypted submit beatmap hash({}) not equal({}).",
                player_name, beatmap_md5, submit_data.beatmap_hash
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

        Some(Self {
            beatmap_md5: data[0].to_string(),
            player_name,
            score_md5: data[2].to_string(),
            n300: try_parse(&data[3])?,
            n100: try_parse(&data[4])?,
            n50: try_parse(&data[5])?,
            geki: try_parse(&data[6])?,
            katu: try_parse(&data[7])?,
            miss: try_parse(&data[8])?,
            score: try_parse(&data[9])?,
            max_combo: try_parse(&data[10])?,
            perfect: &data[11] == "True",
            grade: data[12].to_string(),
            mods,
            pass: &data[14] == "True",
            mode,
            osu_version,
            client_flags,
        })
    }
}
