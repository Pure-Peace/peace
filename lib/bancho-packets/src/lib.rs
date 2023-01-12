#![allow(dead_code)]
#![allow(non_camel_case_types)]

#[cfg(test)]
mod tests;

pub mod client;
pub mod server;

use enum_primitive_derive::Primitive;
use num_traits::FromPrimitive;
use std::{convert::TryInto, ops::Deref};

/// Packet header length
///
/// - 0..=1: packet id
/// - 2: null
/// - 3..=6: packet length
/// - 7..=N: packet data length(uleb128) + data
pub const BANCHO_PACKET_HEADER_LENGTH: usize = 7;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Packet<'a> {
    pub id: PacketId,
    pub payload: Option<&'a [u8]>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct PacketHeader {
    pub id: PacketId,
    pub payload_length: u32,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub enum LoginResult {
    Success(i32),
    Failed(LoginFailedResaon),
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(i32)]
pub enum LoginFailedResaon {
    InvalidCredentials = -1,
    OutdatedClient = -2,
    UserBanned = -3,
    MultiaccountDetected = -4,
    ServerError = -5,
    CuttingEdgeMultiplayer = -6,
    AccountPasswordRest = -7,
    VerificationRequired = -8,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Primitive)]
#[repr(u8)]
pub enum PacketId {
    /// Bancho packet ids
    OSU_USER_CHANGE_ACTION = 0,
    OSU_SEND_PUBLIC_MESSAGE = 1,
    OSU_USER_LOGOUT = 2,
    OSU_USER_REQUEST_STATUS_UPDATE = 3,
    OSU_PING = 4,
    BANCHO_USER_LOGIN_REPLY = 5,
    BANCHO_SEND_MESSAGE = 7,
    BANCHO_PONG = 8,
    BANCHO_HANDLE_IRC_CHANGE_USERNAME = 9,
    BANCHO_HANDLE_IRC_QUIT = 10,
    BANCHO_USER_STATS = 11,
    BANCHO_USER_LOGOUT = 12,
    BANCHO_SPECTATOR_JOINED = 13,
    BANCHO_SPECTATOR_LEFT = 14,
    BANCHO_SPECTATE_FRAMES = 15,
    OSU_SPECTATE_START = 16,
    OSU_SPECTATE_STOP = 17,
    OSU_SPECTATE_FRAMES = 18,
    BANCHO_VERSION_UPDATE = 19,
    OSU_ERROR_REPORT = 20,
    OSU_SPECTATE_CANT = 21,
    BANCHO_SPECTATOR_CANT_SPECTATE = 22,
    BANCHO_GET_ATTENTION = 23,
    BANCHO_NOTIFICATION = 24,
    OSU_SEND_PRIVATE_MESSAGE = 25,
    BANCHO_UPDATE_MATCH = 26,
    BANCHO_NEW_MATCH = 27,
    BANCHO_DISBAND_MATCH = 28,
    OSU_USER_PART_LOBBY = 29,
    OSU_USER_JOIN_LOBBY = 30,
    OSU_USER_CREATE_MATCH = 31,
    OSU_USER_JOIN_MATCH = 32,
    OSU_USER_PART_MATCH = 33,
    BANCHO_TOGGLE_BLOCK_NON_FRIEND_DMS = 34,
    BANCHO_MATCH_JOIN_SUCCESS = 36,
    BANCHO_MATCH_JOIN_FAIL = 37,
    OSU_MATCH_CHANGE_SLOT = 38,
    OSU_USER_MATCH_READY = 39,
    OSU_MATCH_LOCK = 40,
    OSU_MATCH_CHANGE_SETTINGS = 41,
    BANCHO_FELLOW_SPECTATOR_JOINED = 42,
    BANCHO_FELLOW_SPECTATOR_LEFT = 43,
    OSU_MATCH_START = 44,
    BANCHO_ALL_PLAYERS_LOADED = 45,
    BANCHO_MATCH_START = 46,
    OSU_MATCH_SCORE_UPDATE = 47,
    BANCHO_MATCH_SCORE_UPDATE = 48,
    OSU_MATCH_COMPLETE = 49,
    BANCHO_MATCH_TRANSFER_HOST = 50,
    OSU_MATCH_CHANGE_MODS = 51,
    OSU_MATCH_LOAD_COMPLETE = 52,
    BANCHO_MATCH_ALL_PLAYERS_LOADED = 53,
    OSU_MATCH_NO_BEATMAP = 54,
    OSU_MATCH_NOT_READY = 55,
    OSU_MATCH_FAILED = 56,
    BANCHO_MATCH_PLAYER_FAILED = 57,
    BANCHO_MATCH_COMPLETE = 58,
    OSU_MATCH_HAS_BEATMAP = 59,
    OSU_MATCH_SKIP_REQUEST = 60,
    BANCHO_MATCH_SKIP = 61,
    BANCHO_UNAUTHORIZED = 62,
    OSU_USER_CHANNEL_JOIN = 63,
    BANCHO_CHANNEL_JOIN_SUCCESS = 64,
    BANCHO_CHANNEL_INFO = 65,
    BANCHO_CHANNEL_KICK = 66,
    BANCHO_CHANNEL_AUTO_JOIN = 67,
    OSU_BEATMAP_INFO_REQUEST = 68,
    BANCHO_BEATMAP_INFO_REPLY = 69,
    OSU_MATCH_TRANSFER_HOST = 70,
    BANCHO_PRIVILEGES = 71,
    BANCHO_FRIENDS_LIST = 72,
    OSU_USER_FRIEND_ADD = 73,
    OSU_USER_FRIEND_REMOVE = 74,
    BANCHO_PROTOCOL_VERSION = 75,
    BANCHO_MAIN_MENU_ICON = 76,
    OSU_MATCH_CHANGE_TEAM = 77,
    OSU_USER_CHANNEL_PART = 78,
    OSU_USER_RECEIVE_UPDATES = 79,
    BANCHO_MONITOR = 80,
    BANCHO_MATCH_PLAYER_SKIPPED = 81,
    OSU_USER_SET_AWAY_MESSAGE = 82,
    BANCHO_USER_PRESENCE = 83,
    OSU_IRC_ONLY = 84,
    OSU_USER_STATS_REQUEST = 85,
    BANCHO_RESTART = 86,
    OSU_MATCH_INVITE = 87,
    BANCHO_MATCH_INVITE = 88,
    BANCHO_CHANNEL_INFO_END = 89,
    OSU_MATCH_CHANGE_PASSWORD = 90,
    BANCHO_MATCH_CHANGE_PASSWORD = 91,
    BANCHO_SILENCE_END = 92,
    OSU_TOURNAMENT_MATCH_INFO_REQUEST = 93,
    BANCHO_USER_SILENCED = 94,
    BANCHO_USER_PRESENCE_SINGLE = 95,
    BANCHO_USER_PRESENCE_BUNDLE = 96,
    OSU_USER_PRESENCE_REQUEST = 97,
    OSU_USER_PRESENCE_REQUEST_ALL = 98,
    OSU_USER_TOGGLE_BLOCK_NON_FRIEND_DMS = 99,
    BANCHO_USER_DM_BLOCKED = 100,
    BANCHO_TARGET_IS_SILENCED = 101,
    BANCHO_VERSION_UPDATE_FORCED = 102,
    BANCHO_SWITCH_SERVER = 103,
    BANCHO_ACCOUNT_RESTRICTED = 104,
    BANCHO_RTX = 105,
    BANCHO_MATCH_ABORT = 106,
    BANCHO_SWITCH_TOURNAMENT_SERVER = 107,
    OSU_TOURNAMENT_JOIN_MATCH_CHANNEL = 108,
    OSU_TOURNAMENT_LEAVE_MATCH_CHANNEL = 109,
    OSU_UNKNOWN_PACKET = 255,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct BanchoMessage {
    pub sender: String,
    pub content: String,
    pub target: String,
    pub sender_id: i32,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct MatchData {
    pub match_id: i32,
    pub in_progress: bool,
    pub match_type: i8,
    pub play_mods: u32,
    pub match_name: String,
    pub password: Option<String>,
    pub beatmap_name: String,
    pub beatmap_id: i32,
    pub beatmap_md5: String,
    pub slot_status: Vec<u8>,
    pub slot_teams: Vec<u8>,
    pub slot_players: Vec<i32>,
    pub host_player_id: i32,
    pub match_game_mode: u8,
    pub win_condition: u8,
    pub team_type: u8,
    pub freemods: bool,
    pub player_mods: Vec<i32>,
    pub match_seed: i32,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct MatchUpdate {
    pub data: MatchData,
    pub send_password: bool,
}

impl Deref for MatchUpdate {
    type Target = MatchData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct ScoreFrame {
    pub timestamp: i32,
    pub id: u8,
    pub n300: u16,
    pub n100: u16,
    pub n50: u16,
    pub geki: u16,
    pub katu: u16,
    pub miss: u16,
    pub score: i32,
    pub combo: u16,
    pub max_combo: u16,
    pub perfect: bool,
    pub hp: u8,
    pub tag_byte: u8,
    pub score_v2: bool,
}

pub trait BanchoPacketRead<T> {
    fn read(reader: &mut PayloadReader) -> Option<T>;
}

impl BanchoPacketRead<String> for String {
    fn read(reader: &mut PayloadReader) -> Option<String> {
        if reader.payload.get(reader.index())? != &0xb {
            return None
        }
        reader.increase_index(1);
        let data_length = reader.read_uleb128()? as usize;

        let cur = reader.index;
        reader.increase_index(data_length);
        let data = reader.payload.get(cur..reader.index)?;

        Some(std::str::from_utf8(data).ok()?.into())
    }
}

impl BanchoPacketRead<bool> for bool {
    fn read(reader: &mut PayloadReader) -> Option<bool> {
        Some(reader.read::<i8>()? == 1)
    }
}

macro_rules! impl_number {
    ($($t:ty),+) => {
        $(impl BanchoPacketRead<$t> for $t {
            fn read(reader: &mut PayloadReader) -> Option<$t> {
                Some(<$t>::from_le_bytes(
                    reader.next_with_length_type::<$t>()?.try_into().ok()?,
                ))
            }
        })+
    };
}

impl_number!(i8, u8, i16, u16, i32, u32, i64, u64);

impl BanchoPacketRead<BanchoMessage> for BanchoMessage {
    fn read(reader: &mut PayloadReader) -> Option<BanchoMessage> {
        Some(read_struct!(
            reader,
            BanchoMessage { sender, content, target, sender_id }
        ))
    }
}

impl BanchoPacketRead<ScoreFrame> for ScoreFrame {
    fn read(reader: &mut PayloadReader) -> Option<ScoreFrame> {
        Some(read_struct!(
            reader,
            ScoreFrame {
                timestamp,
                id,
                n300,
                n100,
                n50,
                geki,
                katu,
                miss,
                score,
                combo,
                max_combo,
                perfect,
                hp,
                tag_byte,
                score_v2
            }
        ))
    }
}

macro_rules! impl_read_number_array {
    ($($t:ty),+) => {
        $(impl BanchoPacketRead<Vec<$t>> for Vec<$t> {
            fn read(reader: &mut PayloadReader) -> Option<Vec<$t>> {
                let length_data = reader.next_with_length_type::<i16>()?;
                let int_count = <i16>::from_le_bytes(length_data.try_into().ok()?) as usize;

                let mut data = Vec::with_capacity(int_count);
                for _ in 0..int_count {
                    data.push(<$t>::from_le_bytes(reader.next_with_length_type::<i32>()?.try_into().ok()?));
                }
                Some(data)
            }
        })+
    };
}

impl_read_number_array!(i8, u8, i16, u16, i32, u32, i64, u64);

pub trait BanchoPacketWrite {
    fn write_buf(&self, buf: &mut Vec<u8>);
    fn as_packet(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.write_buf(&mut buf);
        buf
    }
}

macro_rules! impl_write_number {
    ($($t:ty),+) => {
        $(impl BanchoPacketWrite for $t {
            #[inline]
            fn write_buf(&self, buf: &mut Vec<u8>) {
                buf.extend(self.to_le_bytes())
            }
        })+
    }
}

macro_rules! impl_write_number_array {
    ($($t:ty),+) => {$(impl BanchoPacketWrite for [$t] {
        #[inline]
        fn write_buf(&self, buf: &mut Vec<u8>) {
            buf.extend((self.len() as u16).to_le_bytes());
            for int in self.iter() {
                buf.extend(int.to_le_bytes())
            }
        }
    })+}
}

impl BanchoPacketWrite for str {
    #[inline]
    fn write_buf(&self, buf: &mut Vec<u8>) {
        let byte_length = self.len();
        if byte_length > 0 {
            buf.push(0xb);
            buf.extend(write_uleb128(byte_length as u32));
            buf.extend(self.as_bytes());
        } else {
            buf.push(0);
        }
    }
}

impl BanchoPacketWrite for u8 {
    #[inline]
    fn write_buf(&self, buf: &mut Vec<u8>) {
        buf.push(*self);
    }
}

impl BanchoPacketWrite for [u8] {
    #[inline]
    fn write_buf(&self, buf: &mut Vec<u8>) {
        buf.extend(self);
    }
}

impl BanchoPacketWrite for bool {
    #[inline]
    fn write_buf(&self, buf: &mut Vec<u8>) {
        buf.push(if *self { 1 } else { 0 });
    }
}

impl_write_number!(i8, u16, i16, i32, u32, i64, u64, f32, f64);
impl_write_number_array!(i8, u16, i16, i32, u32, i64, u64, f32, f64);

impl BanchoPacketWrite for LoginResult {
    #[inline]
    fn write_buf(&self, buf: &mut Vec<u8>) {
        match self {
            LoginResult::Success(user_id) => *user_id,
            LoginResult::Failed(reason) => *reason as i32,
        }
        .write_buf(buf)
    }
}

impl BanchoPacketWrite for MatchUpdate {
    fn write_buf(&self, buf: &mut Vec<u8>) {
        let raw_password = if let Some(pw) = &self.password {
            if self.send_password {
                let mut buf = Vec::new();
                pw.write_buf(&mut buf);
                buf
            } else {
                b"\x0b\x00".to_vec()
            }
        } else {
            b"\x00".to_vec()
        };

        buf.extend(data!(
            self.match_id as u16,
            self.in_progress,
            self.match_type,
            self.play_mods,
            self.match_name,
            raw_password,
            self.beatmap_name,
            self.beatmap_id,
            self.beatmap_md5,
            self.slot_status,
            self.slot_teams,
            self.slot_players,
            self.host_player_id,
            self.match_game_mode,
            self.win_condition,
            self.team_type,
            self.freemods,
            self.player_mods,
            self.match_seed
        ));
    }
}

impl BanchoPacketWrite for MatchData {
    fn write_buf(&self, buf: &mut Vec<u8>) {
        MatchUpdate { data: self.to_owned(), send_password: true }
            .write_buf(buf);
    }
}

impl BanchoPacketWrite for ScoreFrame {
    fn write_buf(&self, buf: &mut Vec<u8>) {
        buf.extend(data!(
            self.timestamp,
            self.id,
            self.n300,
            self.n100,
            self.n50,
            self.geki,
            self.katu,
            self.miss,
            self.score,
            self.combo,
            self.max_combo,
            self.perfect,
            self.hp,
            self.tag_byte,
            self.score_v2
        ));
    }
}

#[derive(Debug, Clone)]
pub struct PayloadReader<'a> {
    pub(crate) payload: &'a [u8],
    pub(crate) index: usize,
}

impl<'a> PayloadReader<'a> {
    #[inline]
    pub fn new(payload: &'a [u8]) -> Self {
        PayloadReader { payload, index: 0 }
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.index
    }

    #[inline]
    pub fn payload(&self) -> &'a [u8] {
        self.payload
    }

    #[inline]
    pub fn reset(&mut self) {
        self.index = 0
    }

    #[inline]
    pub fn increase_index(&mut self, offset: usize) -> usize {
        self.index += offset;
        self.index
    }

    #[inline]
    pub fn decrease_index(&mut self, offset: usize) -> usize {
        self.index -= offset;
        self.index
    }

    #[inline]
    pub(crate) fn next_with_length(&mut self, length: usize) -> Option<&[u8]> {
        self.index += length;
        Some(self.payload.get(self.index - length..self.index)?)
    }

    #[inline]
    pub(crate) fn next_with_length_type<Len: Sized>(
        &mut self,
    ) -> Option<&[u8]> {
        self.next_with_length(std::mem::size_of::<Len>())
    }

    #[inline]
    pub fn read<T>(&mut self) -> Option<T>
    where
        T: BanchoPacketRead<T>,
    {
        T::read(self)
    }

    #[inline]
    pub(crate) fn read_uleb128(&mut self) -> Option<u32> {
        let (val, length) = read_uleb128(&self.payload.get(self.index..)?)?;
        self.index += length;
        Some(val)
    }
}

#[derive(Debug, Clone)]
pub struct PacketReader<'a> {
    buf: &'a [u8],
    index: usize,
    payload_length: usize,
}

impl<'a> PacketReader<'a> {
    #[inline]
    pub fn new(buf: &'a [u8]) -> Self {
        PacketReader { buf, index: 0, payload_length: 0 }
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.index
    }

    #[inline]
    pub fn buffer(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub fn payload_len(&self) -> usize {
        self.payload_length
    }

    #[inline]
    // Reset the packet reader
    pub fn reset(&mut self) {
        self.index = 0;
        self.payload_length = 0;
    }

    #[inline]
    /// Read packet header: (type, length)
    pub fn read_header(header: &[u8]) -> Option<PacketHeader> {
        let id = *header.get(0)?;
        Some(PacketHeader {
            id: PacketId::from_u8(id).unwrap_or(PacketId::OSU_UNKNOWN_PACKET),
            payload_length: u32::from_le_bytes(header[3..=6].try_into().ok()?),
        })
    }
}

impl<'a> Iterator for PacketReader<'a> {
    type Item = Packet<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if (self.buf.len() - self.index) < BANCHO_PACKET_HEADER_LENGTH {
            return None
        }
        // Slice packet header data [u8; 7],
        // including packet id, payload length
        let header =
            &self.buf[self.index..self.index + BANCHO_PACKET_HEADER_LENGTH];
        self.index += BANCHO_PACKET_HEADER_LENGTH;

        // Get packet id and payload length
        let PacketHeader { id, payload_length } =
            PacketReader::read_header(header)?;

        // Read the payload
        let payload = if payload_length == 0 {
            None
        } else {
            self.payload_length = payload_length as usize;
            // Skip this payload at next call
            self.index += self.payload_length;
            self.buf.get(self.index - self.payload_length..self.index)
        };

        Some(Packet { id, payload })
    }
}

#[inline]
/// Unsigned to uleb128
pub fn write_uleb128(mut unsigned: u32) -> Vec<u8> {
    let mut data: Vec<u8> = Vec::with_capacity(2);
    while unsigned >= 0x80 {
        data.push(((unsigned & 0x7f) | 0x80) as u8);
        unsigned >>= 7;
    }
    data.push(unsigned as u8);
    data
}

#[inline]
pub fn read_uleb128(slice: &[u8]) -> Option<(u32, usize)> {
    let (mut val, mut shift, mut index) = (0, 0, 0);
    loop {
        let byte = slice.get(index)?;
        index += 1;
        if (byte & 0x80) == 0 {
            val |= (*byte as u32) << shift;
            return Some((val, index))
        }
        val |= ((byte & 0x7f) as u32) << shift;
        shift += 7;
    }
}

#[inline]
/// Initial a packet with PacketId
///
/// Packets posit:
/// ```ignore
/// [0..=1]: packet id
/// [2]: null
/// [3..=6]: packet length
/// [7..=N]: packet data length(uleb128) + data
/// ```
/// The maximum value of u8 is 255,
///
/// but currently the largest packet id of bancho is only 109,
///
/// so I think it is sufficient to insert the packet_id in the first position
pub fn new_empty_packet(packet_id: PacketId) -> Vec<u8> {
    vec![packet_id as u8, 0, 0, 0, 0, 0, 0]
}

#[inline]
/// Write message packet
pub fn write_message(
    sender: &str,
    sender_id: i32,
    content: &str,
    target: &str,
) -> Vec<u8> {
    data!(sender, content, target, sender_id)
}

#[inline]
pub fn write_channel(name: &str, title: &str, player_count: i16) -> Vec<u8> {
    data!(name, title, player_count)
}

#[derive(Debug, Clone)]
pub struct PacketBuilder {
    buffer: Vec<u8>,
}

impl PacketBuilder {
    #[inline]
    /// Create an empty builder.
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    #[inline]
    /// Create a builder with capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self { buffer: Vec::with_capacity(capacity) }
    }

    #[inline]
    /// Create a builder with [`PacketId`].
    pub fn with_id(packet_id: PacketId) -> Self {
        Self { buffer: new_empty_packet(packet_id) }
    }

    #[inline]
    pub fn from_batch<P, I>(packets: P) -> Self
    where
        I: IntoIterator<Item = u8>,
        P: IntoIterator<Item = I>,
    {
        Self::new().add_batch(packets)
    }

    #[inline]
    pub fn add_batch<P, I>(mut self, packets: P) -> Self
    where
        I: IntoIterator<Item = u8>,
        P: IntoIterator<Item = I>,
    {
        self.add_batch_ref(packets);
        self
    }

    #[inline]
    pub fn add_batch_ref<P, I>(&mut self, packets: P) -> &Self
    where
        I: IntoIterator<Item = u8>,
        P: IntoIterator<Item = I>,
    {
        for i in packets {
            self.buffer.extend(i)
        }
        self
    }

    #[inline]
    pub fn add<P>(mut self, packet: P) -> Self
    where
        P: IntoIterator<Item = u8>,
    {
        self.add_ref(packet);
        self
    }

    #[inline]
    pub fn add_ref<P>(&mut self, packet: P) -> &Self
    where
        P: IntoIterator<Item = u8>,
    {
        self.buffer.extend(packet);
        self
    }

    #[inline]
    pub fn build(self) -> Vec<u8> {
        self.buffer
    }

    #[inline]
    pub fn buffer(&self) -> &Vec<u8> {
        &self.buffer
    }

    #[inline]
    pub fn buffer_mut(&mut self) -> &mut Vec<u8> {
        &mut self.buffer
    }
}

impl From<Vec<u8>> for PacketBuilder {
    fn from(buffer: Vec<u8>) -> Self {
        Self { buffer }
    }
}

pub mod macros {
    #[macro_export]
    macro_rules! read_struct {
    ($reader:ident, $struct:ident { $($field:ident),+ }) => {
        $struct {
            $($field: $reader.read()?,)+
        }
    };
}

    #[macro_export]
    /// Creating osu!packet data
    ///
    /// # Examples:
    /// ```
    /// use bancho_packets::*;
    ///
    /// let val_1: i32 = 123;
    /// let val_2: i16 = 50;
    ///
    /// // Single data, eq with `val_1.write_buf()`
    /// data!(val_1);
    ///
    /// // Mutiple data, default capacity is 30
    /// data!(val_1, val_2);
    ///
    /// // Create packet with capacity
    /// data!(@capacity { 100 }, val_1, val_2);
    /// ```
    macro_rules! data {
    ($item:expr) => {
        {
            let mut buf = Vec::with_capacity(30);
            $item.write_buf(&mut buf);
            buf
        }
    };
    ($($item:expr),+) => {
        {
            let mut buf = Vec::with_capacity(30);
            $($item.write_buf(&mut buf);)+
            buf
        }
    };
    (@capacity { $capacity:expr }, $($item:expr),+) => {
        {
            let mut buf = Vec::with_capacity($capacity);
            $($item.write_buf(&mut buf);)+
            buf
        }
    }
}

    #[macro_export]
    macro_rules! add_packet_length {
        ($packet:expr) => {{
            let packet_length =
                (($packet.len() - $crate::BANCHO_PACKET_HEADER_LENGTH) as i32)
                    .to_le_bytes();
            let ptr = $packet.as_mut_ptr();
            unsafe {
                std::ptr::write(ptr.add(3), packet_length[0]);
                std::ptr::write(ptr.add(4), packet_length[1]);
                std::ptr::write(ptr.add(5), packet_length[2]);
                std::ptr::write(ptr.add(6), packet_length[3]);
            }
            $packet
        }};
    }

    #[macro_export]
    /// Create Bancho packets
    ///
    /// # Examples:
    /// ```
    /// use bancho_packets::*;
    ///
    /// // Origin data here (i32)
    /// let data: i32 = 6;
    /// packet!(PacketId::BANCHO_USER_STATS, data);
    ///
    /// // Packet data here (Vec<u8>)
    /// let data = vec![1, 2, 3];
    /// packet!(PacketId::BANCHO_USER_STATS, data);
    ///
    /// // Only packet_id
    /// packet!(PacketId::BANCHO_USER_STATS);
    ///
    /// // Complex example
    /// let user_id: i32 = 1000;
    /// let username: &str = "username";
    ///
    /// packet!(
    ///     PacketId::BANCHO_USER_PRESENCE,
    ///     data!(
    ///         user_id,
    ///         username
    ///     )
    /// );
    /// ```
    macro_rules! packet {
        ($packet_id:expr) => {
            {
                let mut packet = $crate::new_empty_packet($packet_id);
                $crate::add_packet_length!(packet)
            }
        };
        ($packet_id:expr, $($data:expr),*) => {
            {
                let mut packet = $crate::new_empty_packet($packet_id);
                $($data.write_buf(&mut packet);)*
                $crate::add_packet_length!(packet)
            }
        }
    }
}
