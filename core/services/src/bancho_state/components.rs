use crate::{
    users::{Session, SessionData, UserIndexes, UserStore},
    DumpConfig, DumpData, DumpType,
};
use async_trait::async_trait;
use bancho_packets::server::{UserPresence, UserStats};
use bitmask_enum::bitmask;
use clap_serde_derive::ClapSerde;
use peace_domain::bancho_state::ConnectionInfo;
use std::{collections::VecDeque, sync::Arc};
use tokio::sync::{Mutex, MutexGuard};
use tools::{
    atomic::{Atomic, AtomicOption, AtomicValue, Bool, F32, U32, U64},
    Ulid,
};

pub type PacketData = Vec<u8>;
pub type PacketDataPtr = Arc<Vec<u8>>;

pub type BanchoSession = Session<BanchoExtend>;
pub type BanchoSessionData = SessionData<BanchoExtendData>;

pub type SessionIndexes = UserIndexes<BanchoSession>;
pub type UserSessions = UserStore<BanchoSession>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Packet {
    Data(PacketData),
    Ptr(PacketDataPtr),
}

impl Default for Packet {
    fn default() -> Self {
        Self::Data(Vec::new())
    }
}

impl Packet {
    pub fn new(data: PacketData) -> Self {
        Self::Data(data)
    }

    pub fn new_ptr(data: PacketData) -> Self {
        Self::Ptr(Arc::new(data))
    }
}

impl IntoIterator for Packet {
    type Item = u8;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Packet::Data(data) => data.into_iter(),
            Packet::Ptr(ptr) => Arc::try_unwrap(ptr)
                .unwrap_or_else(|ptr| (*ptr).clone())
                .into_iter(),
        }
    }
}

impl From<Arc<Vec<u8>>> for Packet {
    fn from(ptr: Arc<Vec<u8>>) -> Self {
        Self::Ptr(ptr)
    }
}

impl From<Vec<u8>> for Packet {
    fn from(data: Vec<u8>) -> Self {
        Self::Data(data)
    }
}

#[rustfmt::skip]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Primitive, Hash, Serialize, Deserialize)]
pub enum GameMode {
    #[default]
    Standard            = 0,
    Taiko               = 1,
    Fruits              = 2,
    Mania               = 3,

    StandardRelax       = 4,
    TaikoRelax          = 5,
    FruitsRelax         = 6,
    StandardAutopilot   = 8,

    StandardScoreV2     = 12,
}

impl GameMode {
    #[inline]
    pub fn val(&self) -> u8 {
        *self as u8
    }
}

#[rustfmt::skip]
#[derive(Default)]
#[bitmask(u32)]
pub enum Mods {
    #[default]
    NoMod         = 0,
    NoFail        = 1 << 0,
    Easy          = 1 << 1,
    TouchScreen   = 1 << 2,
    Hidden        = 1 << 3,
    HardRock      = 1 << 4,
    SuddenDeath   = 1 << 5,
    DoubleTime    = 1 << 6,
    Relax         = 1 << 7,
    HalfTime      = 1 << 8,
    NightCore     = 1 << 9,
    FlashLight    = 1 << 10,
    Auto          = 1 << 11,
    SpunOut       = 1 << 12,
    AutoPilot     = 1 << 13,
    Perfect       = 1 << 14,
    Key4          = 1 << 15,
    Key5          = 1 << 16,
    Key6          = 1 << 17,
    Key7          = 1 << 18,
    Key8          = 1 << 19,
    FadeIn        = 1 << 20,
    Random        = 1 << 21,
    Cinema        = 1 << 22,
    Target        = 1 << 23,
    Key9          = 1 << 24,
    KeyCoop       = 1 << 25,
    Key1          = 1 << 26,
    Key3          = 1 << 27,
    Key2          = 1 << 28,
    ScoreV2       = 1 << 29,
    Mirror        = 1 << 30,

    KeyMods = Self::Key1.bits
        | Self::Key2.bits
        | Self::Key3.bits
        | Self::Key4.bits
        | Self::Key5.bits
        | Self::Key6.bits
        | Self::Key7.bits
        | Self::Key8.bits
        | Self::Key9.bits,

    ScoreIncrease = Self::Hidden.bits
        | Self::HardRock.bits
        | Self::FadeIn.bits
        | Self::DoubleTime.bits
        | Self::FlashLight.bits,

    SpeedChanging =
        Self::DoubleTime.bits | Self::NightCore.bits | Self::HalfTime.bits,

    StandardOnly = Self::AutoPilot.bits | Self::SpunOut.bits | Self::Target.bits,
    ManiaOnly = Self::Mirror.bits
        | Self::Random.bits
        | Self::FadeIn.bits
        | Self::KeyMods.bits,
}

impl serde::Serialize for Mods {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(self.bits())
    }
}

impl<'de> serde::Deserialize<'de> for Mods {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        u32::deserialize(deserializer).map(Self::from)
    }
}

#[rustfmt::skip]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Primitive, Serialize, Deserialize)]
pub enum UserOnlineStatus {
    #[default]
    Idle          = 0,
    Afk           = 1,
    Playing       = 2,
    Editing       = 3,
    Modding       = 4,
    Multiplayer   = 5,
    Watching      = 6,
    Unknown       = 7,
    Testing       = 8,
    Submitting    = 9,
    Paused        = 10,
    Lobby         = 11,
    Multiplaying  = 12,
    Direct        = 13,
}

impl UserOnlineStatus {
    #[inline]
    pub fn val(&self) -> u8 {
        *self as u8
    }
}

#[rustfmt::skip]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Primitive, Serialize, Deserialize)]
pub enum PresenceFilter {
    #[default]
    None    = 0,
    All     = 1,
    Friends = 2,
}

impl PresenceFilter {
    #[inline]
    pub fn val(&self) -> i32 {
        *self as i32
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModeStats {
    pub rank: U32,
    pub pp_v2: F32,
    pub accuracy: F32,
    pub total_hits: U32,
    pub total_score: U64,
    pub ranked_score: U64,
    pub playcount: U32,
    pub playtime: U64,
    pub max_combo: U32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BanchoStatus {
    pub online_status: Atomic<UserOnlineStatus>,
    pub description: Atomic<String>,
    pub beatmap_id: U32,
    pub beatmap_md5: Atomic<String>,
    pub mods: Atomic<Mods>,
    pub mode: Atomic<GameMode>,
}

impl BanchoStatus {
    #[inline]
    pub fn update_all(
        &self,
        online_status: UserOnlineStatus,
        description: String,
        beatmap_id: u32,
        beatmap_md5: String,
        mods: Mods,
        mode: GameMode,
    ) {
        self.online_status.set(online_status.into());
        self.description.set(description.into());
        self.beatmap_id.set(beatmap_id);
        self.beatmap_md5.set(beatmap_md5.into());
        self.mods.set(mods.into());
        self.mode.set(mode.into());
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserModeStatSets {
    pub standard: AtomicOption<ModeStats>,
    pub taiko: AtomicOption<ModeStats>,
    pub fruits: AtomicOption<ModeStats>,
    pub mania: AtomicOption<ModeStats>,
    pub standard_relax: AtomicOption<ModeStats>,
    pub taiko_relax: AtomicOption<ModeStats>,
    pub fruits_relax: AtomicOption<ModeStats>,
    pub standard_autopilot: AtomicOption<ModeStats>,
    pub standard_score_v2: AtomicOption<ModeStats>,
}

#[derive(Debug, Clone, Default)]
pub struct BanchoPacketsQueue {
    pub queue: Arc<Mutex<VecDeque<Packet>>>,
}

impl From<Vec<Packet>> for BanchoPacketsQueue {
    fn from(packets: Vec<Packet>) -> Self {
        let mut queue = VecDeque::with_capacity(packets.len());
        for p in packets {
            queue.push_back(p);
        }
        Self::new(queue)
    }
}

impl From<Vec<u8>> for BanchoPacketsQueue {
    fn from(packets: Vec<u8>) -> Self {
        Self::new(VecDeque::from([packets.into()]))
    }
}

impl BanchoPacketsQueue {
    #[inline]
    pub fn new(packets: VecDeque<Packet>) -> Self {
        Self { queue: Arc::new(Mutex::new(packets)) }
    }

    #[inline]
    pub async fn queued_packets(&self) -> usize {
        self.queue.lock().await.len()
    }

    #[inline]
    pub async fn push_packet(&self, packet: Packet) -> usize {
        let mut queue = self.queue.lock().await;
        queue.push_back(packet);
        queue.len()
    }

    #[inline]
    pub async fn enqueue_packets<I>(&self, packets: I) -> usize
    where
        I: IntoIterator<Item = Packet>,
    {
        let mut queue = self.queue.lock().await;
        queue.extend(packets);
        queue.len()
    }

    #[inline]
    pub async fn dequeue_packet(
        &self,
        queue_lock: Option<&mut MutexGuard<'_, VecDeque<Packet>>>,
    ) -> Option<Packet> {
        match queue_lock {
            Some(queue) => queue.pop_front(),
            None => self.queue.lock().await.pop_front(),
        }
    }

    #[inline]
    pub async fn dequeue_all_packets(
        &self,
        queue_lock: Option<&mut MutexGuard<'_, VecDeque<Packet>>>,
    ) -> Vec<u8> {
        let mut buf = Vec::new();

        #[inline]
        fn dequeue(
            buf: &mut Vec<u8>,
            queue_lock: &mut MutexGuard<'_, VecDeque<Packet>>,
        ) {
            while let Some(packet) = queue_lock.pop_front() {
                buf.extend(packet);
            }
        }

        match queue_lock {
            Some(queue_lock) => dequeue(&mut buf, queue_lock),
            None => dequeue(&mut buf, &mut self.queue.lock().await),
        };

        buf
    }

    pub async fn dump_packets(&self) -> Vec<Packet> {
        let queue = self.queue.lock().await;
        Vec::from_iter(queue.iter().cloned())
    }
}

impl serde::Serialize for BanchoPacketsQueue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let packets = self.queue.blocking_lock().clone();
        packets.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for BanchoPacketsQueue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::new(VecDeque::deserialize(deserializer)?))
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BanchoExtend {
    pub client_version: String,
    pub utc_offset: u8,
    pub presence_filter: Atomic<PresenceFilter>,
    pub display_city: bool,
    pub only_friend_pm_allowed: Bool,
    pub bancho_status: BanchoStatus,
    pub mode_stat_sets: UserModeStatSets,
    pub packets_queue: BanchoPacketsQueue,
    /// Information about the user's connection.
    pub connection_info: ConnectionInfo,
    pub notify_index: Atomic<Ulid>,
}

impl From<BanchoExtendData> for BanchoExtend {
    fn from(data: BanchoExtendData) -> Self {
        Self {
            client_version: data.client_version,
            utc_offset: data.utc_offset,
            presence_filter: data.presence_filter.into(),
            display_city: data.display_city,
            only_friend_pm_allowed: data.only_friend_pm_allowed.into(),
            bancho_status: data.bancho_status,
            mode_stat_sets: data.mode_stat_sets,
            packets_queue: data.packets_queue.into(),
            connection_info: data.connection_info,
            notify_index: data.notify_index.into(),
        }
    }
}

impl BanchoExtend {
    #[inline]
    pub fn new(
        client_version: String,
        utc_offset: u8,
        display_city: bool,
        only_friend_pm_allowed: impl Into<Bool>,
        initial_packets: Option<Vec<u8>>,
        connection_info: ConnectionInfo,
    ) -> Self {
        Self {
            client_version,
            utc_offset,
            display_city,
            only_friend_pm_allowed: only_friend_pm_allowed.into(),
            packets_queue: initial_packets
                .map(BanchoPacketsQueue::from)
                .unwrap_or_default(),
            connection_info,
            ..Default::default()
        }
    }
}

impl Session<BanchoExtend> {
    #[inline]
    pub fn mode_stats(&self) -> Option<Arc<ModeStats>> {
        let stats = &self.extends.mode_stat_sets;
        match &self.extends.bancho_status.mode.load().as_ref() {
            GameMode::Standard => stats.standard.load_full(),
            GameMode::Taiko => stats.taiko.load_full(),
            GameMode::Fruits => stats.fruits.load_full(),
            GameMode::Mania => stats.mania.load_full(),
            GameMode::StandardRelax => stats.standard_relax.load_full(),
            GameMode::TaikoRelax => stats.taiko_relax.load_full(),
            GameMode::FruitsRelax => stats.fruits_relax.load_full(),
            GameMode::StandardAutopilot => stats.standard_autopilot.load_full(),
            GameMode::StandardScoreV2 => stats.standard_score_v2.load_full(),
        }
    }

    #[inline]
    pub fn user_info_packets(&self) -> Vec<u8> {
        let mut info = self.user_stats_packet();
        info.extend(self.user_presence_packet());
        info
    }

    #[inline]
    pub fn user_stats_packet(&self) -> Vec<u8> {
        let status = &self.extends.bancho_status;
        let stats = self.mode_stats();
        let stats = stats.as_deref();

        UserStats::pack(
            self.user_id,
            status.online_status.load().val(),
            status.description.to_string().into(),
            status.beatmap_md5.to_string().into(),
            status.mods.load().bits(),
            status.mode.load().val(),
            status.beatmap_id.val() as i32,
            stats.map(|s| s.ranked_score.val()).unwrap_or_default() as i64,
            stats.map(|s| s.accuracy.val()).unwrap_or_default(),
            stats.map(|s| s.playcount.val()).unwrap_or_default() as i32,
            stats.map(|s| s.total_score.val()).unwrap_or_default() as i64,
            stats.map(|s| s.rank.val()).unwrap_or_default() as i32,
            stats.map(|s| s.pp_v2.val() as i16).unwrap_or_default(),
        )
    }

    #[inline]
    pub fn user_presence_packet(&self) -> Vec<u8> {
        UserPresence::pack(
            self.user_id,
            self.username.to_string().into(),
            self.extends.utc_offset,
            0, // todo
            1, // todo
            self.extends.connection_info.location.longitude as f32,
            self.extends.connection_info.location.latitude as f32,
            self.mode_stats().map(|s| s.rank.val()).unwrap_or_default() as i32,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanchoExtendData {
    pub client_version: String,
    pub utc_offset: u8,
    pub presence_filter: PresenceFilter,
    pub display_city: bool,
    pub only_friend_pm_allowed: bool,
    pub bancho_status: BanchoStatus,
    pub mode_stat_sets: UserModeStatSets,
    pub packets_queue: Vec<Packet>,
    pub connection_info: ConnectionInfo,
    pub notify_index: Ulid,
}

#[async_trait]
impl DumpData<BanchoExtendData> for BanchoExtend {
    async fn dump_data(&self) -> BanchoExtendData {
        BanchoExtendData {
            client_version: self.client_version.clone(),
            utc_offset: self.utc_offset,
            presence_filter: *self.presence_filter.load().as_ref(),
            display_city: self.display_city,
            only_friend_pm_allowed: self.only_friend_pm_allowed.val(),
            bancho_status: self.bancho_status.clone(),
            mode_stat_sets: self.mode_stat_sets.clone(),
            packets_queue: self.packets_queue.dump_packets().await,
            connection_info: self.connection_info.clone(),
            notify_index: *self.notify_index.load().as_ref(),
        }
    }
}

#[derive(Debug, Clone, Parser, ClapSerde, Serialize, Deserialize)]
pub struct CliBanchoStateServiceDumpConfigs {
    #[default("./.service_dump/bancho_state.dump".to_owned())]
    #[arg(long, default_value = "./.service_dump/bancho_state.dump")]
    pub bancho_state_dump_path: String,

    #[default(DumpType::Binary)]
    #[arg(long, value_enum, default_value = "binary")]
    pub bancho_state_dump_type: DumpType,

    #[arg(long)]
    pub bancho_state_save_dump: bool,

    #[arg(long)]
    pub bancho_state_load_dump: bool,

    #[default(300)]
    #[arg(long, default_value = "300")]
    pub bancho_state_dump_expires: u64,
}

impl DumpConfig for CliBanchoStateServiceDumpConfigs {
    fn dump_path(&self) -> &str {
        &self.bancho_state_dump_path
    }

    fn dump_type(&self) -> DumpType {
        self.bancho_state_dump_type
    }

    fn save_dump(&self) -> bool {
        self.bancho_state_save_dump
    }

    fn load_dump(&self) -> bool {
        self.bancho_state_load_dump
    }

    fn dump_expires(&self) -> u64 {
        self.bancho_state_dump_expires
    }
}
