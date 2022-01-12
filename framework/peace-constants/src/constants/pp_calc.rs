use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PPRaw {
    pub acc: Option<f32>,
    pub aim: Option<f32>,
    pub spd: Option<f32>,
    pub str: Option<f32>,
    pub total: f32,
}

#[derive(Debug, Deserialize)]
pub struct PPAcclist {
    pub a95: f32,
    pub a98: f32,
    pub a99: f32,
    pub a100: f32,
}

#[derive(Debug, Deserialize)]
pub struct PPCalcResult {
    pub message: String,
    pub status: i32,
    pub mode: u8,
    pub mods: u32,
    pub pp: f32,
    pub stars: f32,
    pub raw: Option<PPRaw>,
    pub acc_list: Option<PPAcclist>,
}
