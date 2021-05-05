use std::sync::Arc;

use async_std::sync::RwLock;
use chrono::{DateTime, Local};
use enum_primitive_derive::Primitive;
use peace_constants::{GameMode, PlayMods};
use peace_objects::beatmaps::Beatmap;

use crate::objects::{Channel, Player};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Primitive)]
#[repr(u8)]
pub enum MatchSlotStatus {
    Open = 1,
    Locked = 2,
    NotReady = 4,
    Ready = 8,
    NoMap = 16,
    Playing = 32,
    Complete = 64,
    HasPlayer = 124,
    Quit = 128,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Primitive)]
#[repr(u8)]
pub enum MatchTeams {
    Non = 0,
    Blue = 1,
    Red = 2,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Primitive)]
#[repr(u8)]
pub enum MatchGoal {
    Score = 0,
    Accuracy = 1,
    Combo = 2,
    Scorev2 = 3,
    // TODO: support for pp
    PPv2 = 4,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Primitive)]
#[repr(u8)]
pub enum MatchTeamMode {
    HeadToHead = 0,
    TagCoop = 1,
    TeamVs = 2,
    TagTeamVs = 3,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Primitive)]
#[repr(u8)]
pub enum MatchStatus {
    Idle = 0,
    AllReady = 1,
    Starting = 2,
    InProgress = 3,
    WaitingComplete = 4,
}

pub struct Tourney {
    pub id: i32,
    pub name: String,
    pub game_id: i32,
    pub game_name: String,
    pub game_size: i16,
    pub referees: Vec<i32>,
    pub streamers: Vec<i32>,
    pub eligible_players: Vec<i32>,
}

pub struct MappoolItem {
    pub group: String,
    pub code: String,
    pub note: String,
    pub picker_id: i32,
    pub picker: String,
    pub mods: PlayMods,
    pub mode: GameMode,
    pub beatmap: Option<Beatmap>,
}

pub struct Mappool {
    pub id: i32,
    pub name: String,
    pub stage: String,
    pub items: Vec<MappoolItem>,
    pub bans: Vec<i32>,
    pub creator_id: i32,
    pub creator: String,
    pub update_time: DateTime<Local>,
}

pub struct MatchSlot {
    pub player: Option<Arc<RwLock<Player>>>,
    pub status: MatchSlotStatus,
    pub mods: PlayMods,
    pub team: MatchTeams,
    pub loaded: bool,
    pub skipped: bool,
    pub completed: bool,
}

impl MatchSlot {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            player: None,
            status: MatchSlotStatus::Open,
            mods: PlayMods::new(),
            team: MatchTeams::Non,
            loaded: false,
            skipped: false,
            completed: false,
        }
    }

    #[inline(always)]
    pub fn reset(&mut self) {
        self.player = None;
        self.status = MatchSlotStatus::Open;
        self.mods = PlayMods::new();
        self.team = MatchTeams::Non;
        self.loaded = false;
        self.skipped = false;
        self.completed = false;
    }

    #[inline(always)]
    pub fn make_slots(size: i32) -> [Self; 16] {
        let size = if size > 16 {
            16
        } else if size < 1 {
            1
        } else {
            size
        };
        let mut s: [Self; 16] = array_init::array_init(|_| Self::new());
        let mut close = 16 - size;
        let mut index = 15;
        while close > 0 {
            s[index].status = MatchSlotStatus::Locked;
            index -= 1;
            close -= 1;
        }
        s
    }
}

pub struct Match {
    pub id: i64,
    pub name: String,
    pub password: Option<String>,
    pub status: MatchStatus,
    pub slots: [MatchSlot; 16],

    pub is_tourney: bool,
    pub is_temp: bool,
    pub is_locked: bool,

    pub tourney: Option<Tourney>,
    pub mappool: Option<Mappool>,

    pub host_id: i32,
    pub referees: Vec<i32>,
    pub streamers: Vec<i32>,

    pub beatmap_id: i32,
    pub beatmap_md5: String,
    pub beatmap_name: String,

    pub mods: PlayMods,
    pub mode: GameMode,
    pub free_mods: bool,

    pub channel: Arc<RwLock<Channel>>,
    pub team_mode: MatchTeamMode,
    pub goal: MatchGoal,

    pub random_seed: i32,

    pub create_time: DateTime<Local>,
    pub last_update: DateTime<Local>,
}

impl Match {
    #[inline(always)]
    pub fn new(
        id: i64,
        name: String,
        password: Option<String>,
        size: i32,
        host_id: i32,
        is_tourney: bool,
        channel: Arc<RwLock<Channel>>
    ) -> Self {
        let now = Local::now();
        Self {
            id,
            name,
            password,
            status: MatchStatus::Idle,
            slots: MatchSlot::make_slots(size),
            is_tourney,
            is_temp: is_tourney,
            is_locked: false,
            tourney: None,
            mappool: None,
            host_id,
            referees: Vec::new(),
            streamers: Vec::new(),
            beatmap_id: -1,
            beatmap_md5: String::new(),
            beatmap_name: String::new(),
            mods: PlayMods::new(),
            mode: GameMode::Std,
            free_mods: true,
            channel,
            team_mode: MatchTeamMode::HeadToHead,
            goal: MatchGoal::Score,
            random_seed: 0,
            create_time: now,
            last_update: now,
        }
    }

    #[inline(always)]
    pub fn invite_link(&self) -> String {
        let mut link = format!("osump://{}/", self.id);
        if let Some(pw) = &self.password {
            link += pw;
        }
        link
    }

    #[inline(always)]
    pub fn map_bid_url(&self, base_url: Option<&str>) -> String {
        format!(
            "{}/b/{}",
            base_url.unwrap_or("https://osu.ppy.sh"),
            self.beatmap_id
        )
    }

    #[inline(always)]
    pub fn map_md5_url(&self, base_url: Option<&str>) -> String {
        format!(
            "{}/h/{}",
            base_url.unwrap_or("https://osu.ppy.sh"),
            self.beatmap_md5
        )
    }
}
