#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]

#[cfg(test)]
mod tests;

/// Predefined client related bancho packets.
pub mod client;
/// Predefined server related bancho packets.
pub mod server;

#[cfg(feature = "derive")]
pub use bancho_packets_derive::*;

use enum_primitive_derive::Primitive;
use num_traits::FromPrimitive;
use std::{
    convert::TryInto,
    ops::{Deref, DerefMut},
};

/// Packet header length
///
/// - 0..=1: packet id
/// - 2: null
/// - 3..=6: packet length
/// - 7..=N: packet data length(uleb128) + data
pub const BANCHO_PACKET_HEADER_LENGTH: usize = 7;

pub const EMPTY_STRING_PACKET: &[u8; 2] = b"\x0b\x00";

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
/// Bancho packet, including [`PacketHeader`] structure and payload bytes.
pub struct Packet<'a> {
    pub id: PacketId,
    pub payload: Option<&'a [u8]>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
/// Bancho packet header, including [`PacketId`] and payload (data) length.
pub struct PacketHeader {
    pub id: PacketId,
    pub payload_length: u32,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
/// The login result of the bancho client.
/// Returns `Success(user_id)` if success.
pub enum LoginResult {
    Success(i32),
    Failed(LoginFailedResaon),
}

#[rustfmt::skip]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(i32)]
/// The bancho client will handle these failure reasons when the user login fails.
pub enum LoginFailedResaon {
    InvalidCredentials        = -1,
    OutdatedClient            = -2,
    UserBanned                = -3,
    MultiaccountDetected      = -4,
    ServerError               = -5,
    CuttingEdgeMultiplayer    = -6,
    AccountPasswordRest       = -7,
    VerificationRequired      = -8,
}

#[rustfmt::skip]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Primitive)]
#[repr(u8)]
/// Known packet ids for bancho clients.
pub enum PacketId {
    OSU_USER_CHANGE_ACTION                = 0,
    OSU_SEND_PUBLIC_MESSAGE               = 1,
    OSU_USER_LOGOUT                       = 2,
    OSU_USER_REQUEST_STATUS_UPDATE        = 3,
    OSU_PING                              = 4,
    BANCHO_USER_LOGIN_REPLY               = 5,
    BANCHO_SEND_MESSAGE                   = 7,
    BANCHO_PONG                           = 8,
    BANCHO_HANDLE_IRC_CHANGE_USERNAME     = 9,
    BANCHO_HANDLE_IRC_QUIT                = 10,
    BANCHO_USER_STATS                     = 11,
    BANCHO_USER_LOGOUT                    = 12,
    BANCHO_SPECTATOR_JOINED               = 13,
    BANCHO_SPECTATOR_LEFT                 = 14,
    BANCHO_SPECTATE_FRAMES                = 15,
    OSU_SPECTATE_START                    = 16,
    OSU_SPECTATE_STOP                     = 17,
    OSU_SPECTATE_FRAMES                   = 18,
    BANCHO_VERSION_UPDATE                 = 19,
    OSU_ERROR_REPORT                      = 20,
    OSU_SPECTATE_CANT                     = 21,
    BANCHO_SPECTATOR_CANT_SPECTATE        = 22,
    BANCHO_GET_ATTENTION                  = 23,
    BANCHO_NOTIFICATION                   = 24,
    OSU_SEND_PRIVATE_MESSAGE              = 25,
    BANCHO_UPDATE_MATCH                   = 26,
    BANCHO_NEW_MATCH                      = 27,
    BANCHO_DISBAND_MATCH                  = 28,
    OSU_USER_PART_LOBBY                   = 29,
    OSU_USER_JOIN_LOBBY                   = 30,
    OSU_USER_CREATE_MATCH                 = 31,
    OSU_USER_JOIN_MATCH                   = 32,
    OSU_USER_PART_MATCH                   = 33,
    BANCHO_TOGGLE_BLOCK_NON_FRIEND_DMS    = 34,
    BANCHO_MATCH_JOIN_SUCCESS             = 36,
    BANCHO_MATCH_JOIN_FAIL                = 37,
    OSU_MATCH_CHANGE_SLOT                 = 38,
    OSU_USER_MATCH_READY                  = 39,
    OSU_MATCH_LOCK                        = 40,
    OSU_MATCH_CHANGE_SETTINGS             = 41,
    BANCHO_FELLOW_SPECTATOR_JOINED        = 42,
    BANCHO_FELLOW_SPECTATOR_LEFT          = 43,
    OSU_MATCH_START                       = 44,
    BANCHO_ALL_PLAYERS_LOADED             = 45,
    BANCHO_MATCH_START                    = 46,
    OSU_MATCH_SCORE_UPDATE                = 47,
    BANCHO_MATCH_SCORE_UPDATE             = 48,
    OSU_MATCH_COMPLETE                    = 49,
    BANCHO_MATCH_TRANSFER_HOST            = 50,
    OSU_MATCH_CHANGE_MODS                 = 51,
    OSU_MATCH_LOAD_COMPLETE               = 52,
    BANCHO_MATCH_ALL_PLAYERS_LOADED       = 53,
    OSU_MATCH_NO_BEATMAP                  = 54,
    OSU_MATCH_NOT_READY                   = 55,
    OSU_MATCH_FAILED                      = 56,
    BANCHO_MATCH_PLAYER_FAILED            = 57,
    BANCHO_MATCH_COMPLETE                 = 58,
    OSU_MATCH_HAS_BEATMAP                 = 59,
    OSU_MATCH_SKIP_REQUEST                = 60,
    BANCHO_MATCH_SKIP                     = 61,
    BANCHO_UNAUTHORIZED                   = 62,
    OSU_USER_CHANNEL_JOIN                 = 63,
    BANCHO_CHANNEL_JOIN_SUCCESS           = 64,
    BANCHO_CHANNEL_INFO                   = 65,
    BANCHO_CHANNEL_KICK                   = 66,
    BANCHO_CHANNEL_AUTO_JOIN              = 67,
    OSU_BEATMAP_INFO_REQUEST              = 68,
    BANCHO_BEATMAP_INFO_REPLY             = 69,
    OSU_MATCH_TRANSFER_HOST               = 70,
    BANCHO_PRIVILEGES                     = 71,
    BANCHO_FRIENDS_LIST                   = 72,
    OSU_USER_FRIEND_ADD                   = 73,
    OSU_USER_FRIEND_REMOVE                = 74,
    BANCHO_PROTOCOL_VERSION               = 75,
    BANCHO_MAIN_MENU_ICON                 = 76,
    OSU_MATCH_CHANGE_TEAM                 = 77,
    OSU_USER_CHANNEL_PART                 = 78,
    OSU_USER_RECEIVE_UPDATES              = 79,
    BANCHO_MONITOR                        = 80,
    BANCHO_MATCH_PLAYER_SKIPPED           = 81,
    OSU_USER_SET_AWAY_MESSAGE             = 82,
    BANCHO_USER_PRESENCE                  = 83,
    OSU_IRC_ONLY                          = 84,
    OSU_USER_STATS_REQUEST                = 85,
    BANCHO_RESTART                        = 86,
    OSU_MATCH_INVITE                      = 87,
    BANCHO_MATCH_INVITE                   = 88,
    BANCHO_CHANNEL_INFO_END               = 89,
    OSU_MATCH_CHANGE_PASSWORD             = 90,
    BANCHO_MATCH_CHANGE_PASSWORD          = 91,
    BANCHO_SILENCE_END                    = 92,
    OSU_TOURNAMENT_MATCH_INFO_REQUEST     = 93,
    BANCHO_USER_SILENCED                  = 94,
    BANCHO_USER_PRESENCE_SINGLE           = 95,
    BANCHO_USER_PRESENCE_BUNDLE           = 96,
    OSU_USER_PRESENCE_REQUEST             = 97,
    OSU_USER_PRESENCE_REQUEST_ALL         = 98,
    OSU_USER_TOGGLE_BLOCK_NON_FRIEND_DMS  = 99,
    BANCHO_USER_DM_BLOCKED                = 100,
    BANCHO_TARGET_IS_SILENCED             = 101,
    BANCHO_VERSION_UPDATE_FORCED          = 102,
    BANCHO_SWITCH_SERVER                  = 103,
    BANCHO_ACCOUNT_RESTRICTED             = 104,
    BANCHO_RTX                            = 105,
    BANCHO_MATCH_ABORT                    = 106,
    BANCHO_SWITCH_TOURNAMENT_SERVER       = 107,
    OSU_TOURNAMENT_JOIN_MATCH_CHANNEL     = 108,
    OSU_TOURNAMENT_LEAVE_MATCH_CHANNEL    = 109,
    OSU_UNKNOWN_PACKET                    = 255,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, ReadPacket)]
/// [`BanchoMessage`] is the message structure of the bancho client.
pub struct BanchoMessage {
    pub sender: String,
    pub content: String,
    pub target: String,
    pub sender_id: i32,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PacketLength)]
/// [`MatchData`] is the data of bancho client multiplayer game room.
pub struct MatchData {
    pub match_id: i32,
    pub in_progress: bool,
    pub match_type: i8,
    pub play_mods: u32,
    pub match_name: String,
    #[length(self.password.as_ref().map(|pw| pw.packet_len()).unwrap_or(2))]
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
#[derive(Debug, Clone, PacketLength)]
pub struct MatchUpdate {
    pub data: MatchData,
    pub send_password: bool,
}

impl Deref for MatchUpdate {
    type Target = MatchData;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for MatchUpdate {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, ReadPacket, WritePacket, PacketLength)]
/// The [`ScoreFrame`] uploaded by the bancho client during multiplayer games.
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

/// Generic type for [`str`] and [`String`]
pub trait Str:
    BanchoPacketWrite
    + BanchoPacketLength
    + std::fmt::Display
    + std::fmt::Debug
    + AsRef<str>
    + Into<String>
    + Deref<Target = str>
{
}

impl Str for &str {}

impl Str for String {}

#[derive(Debug, Clone)]
/// [`PayloadReader`] helps to read Bacho packet data.
///
/// ### Usage:
/// ```
/// use bancho_packets::{PacketReader, PayloadReader};
///
///
/// let mut reader = PacketReader::new(&[
///     4, 0, 0, 0, 0, 0, 0, 24, 0, 0, 19, 0, 0, 0, 11, 17, 72, 101, 108,
///     108, 111, 44, 32, 87, 111, 114, 108, 100, 33, 240, 159, 146, 150,
///     4, 0, 0, 0, 0, 0, 0, 24, 0, 0, 18, 0, 0, 0, 11, 16, 229, 147, 136,
///     229, 147, 136, 227, 128, 144, 240, 159, 152, 131, 227, 128, 145,
///     104, 0, 0, 0, 0, 0, 0, 24, 0, 0, 23, 0, 0, 0, 11, 21, 232, 175,
///     187, 229, 143, 150, 229, 174, 140, 228, 186, 134, 239, 188, 129,
///     239, 188, 129, 226, 156, 168,
/// ]);
/// while let Some(packet) = reader.next() {
///     print!("packet id: {:?}: ", packet.id);
///     match packet.payload {
///         None => println!("Non-payload"),
///         Some(payload) => {
///             let mut payload_reader = PayloadReader::new(payload);
///             println!("payload as string: {:?}", payload_reader.read::<String>());
///         },
///     }
/// }
///
/// ```
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
    /// Try to read `<T>` from payloads.
    /// Returns [`None`] when no data matching the given type can be read.
    pub fn read<T>(&mut self) -> Option<T>
    where
        T: BanchoPacketRead<T>,
    {
        T::read(self)
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
    /// Reset read progress to `0`.
    pub fn reset(&mut self) {
        self.index = 0
    }

    #[inline]
    pub(crate) fn increase_index(&mut self, offset: usize) -> usize {
        self.index += offset;
        self.index
    }

    #[inline]
    pub(crate) fn decrease_index(&mut self, offset: usize) -> usize {
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
    pub(crate) fn read_uleb128(&mut self) -> Option<u32> {
        let (val, length) = uleb128_to_u32(&self.payload.get(self.index..)?)?;
        self.index += length;
        Some(val)
    }
}

#[derive(Debug, Clone)]
/// [`PacketReader`] helps to read Bacho packets.
///
/// ### Usage:
/// ```
/// use bancho_packets::{PacketReader, PayloadReader};
///
/// // Parsing [`PacketHeader`] from bytes.
/// let header = PacketReader::parse_header(&[
///     24, 0, 0, 7, 0, 0, 0, 11, 5, 104, 101, 108, 108, 111
/// ]);
///
/// // Read sequentially.
/// let mut reader = PacketReader::new(&[
///     24, 0, 0, 7, 0, 0, 0, 11, 5, 104, 101, 108, 108, 111, 4, 0, 0, 0,
///     0, 0, 0, 24, 0, 0, 7, 0, 0, 0, 11, 5, 104, 101, 108, 108, 111,
/// ]);
/// println!("packet 0: {:?}", reader.next());
/// println!("packet 1: {:?}", reader.next());
/// println!("packet 2: {:?}", reader.next());
/// println!("packet 3 (outside): {:?}", reader.next());
///
/// // Full example.
/// let mut reader = PacketReader::new(&[
///     4, 0, 0, 0, 0, 0, 0, 24, 0, 0, 19, 0, 0, 0, 11, 17, 72, 101, 108,
///     108, 111, 44, 32, 87, 111, 114, 108, 100, 33, 240, 159, 146, 150,
///     4, 0, 0, 0, 0, 0, 0, 24, 0, 0, 18, 0, 0, 0, 11, 16, 229, 147, 136,
///     229, 147, 136, 227, 128, 144, 240, 159, 152, 131, 227, 128, 145,
///     104, 0, 0, 0, 0, 0, 0, 24, 0, 0, 23, 0, 0, 0, 11, 21, 232, 175,
///     187, 229, 143, 150, 229, 174, 140, 228, 186, 134, 239, 188, 129,
///     239, 188, 129, 226, 156, 168,
/// ]);
/// while let Some(packet) = reader.next() {
///     print!("packet id: {:?}: ", packet.id);
///     match packet.payload {
///         None => println!("Non-payload"),
///         Some(payload) => {
///             let mut payload_reader = PayloadReader::new(payload);
///             println!("payload as string: {:?}", payload_reader.read::<String>());
///         },
///     }
/// }
///
/// ```
pub struct PacketReader<'a> {
    buf: &'a [u8],
    ptr: usize,
}

impl<'a> PacketReader<'a> {
    #[inline]
    /// Create a new [`PacketReader`] with bytes.
    pub fn new(buf: &'a [u8]) -> Self {
        PacketReader { buf, ptr: 0 }
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.ptr
    }

    #[inline]
    pub fn buffer(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    /// Reset the reading progress of [`PacketReader`].
    pub fn reset(&mut self) {
        self.ptr = 0;
    }

    #[inline]
    /// Parse [`PacketHeader`] from bytes.
    pub fn parse_header(header: &[u8]) -> Option<PacketHeader> {
        let packet_id = *header.get(0)?;
        Some(PacketHeader {
            id: PacketId::from_u8(packet_id)
                .unwrap_or(PacketId::OSU_UNKNOWN_PACKET),
            payload_length: u32::from_le_bytes(
                header[3..BANCHO_PACKET_HEADER_LENGTH].try_into().ok()?,
            ),
        })
    }
}

impl<'a> Iterator for PacketReader<'a> {
    type Item = Packet<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if (self.buf.len() - self.ptr) < BANCHO_PACKET_HEADER_LENGTH {
            return None;
        }
        // Slice packet header data `[u8; 7]`,
        // including packet id, payload length
        let header =
            &self.buf[self.ptr..self.ptr + BANCHO_PACKET_HEADER_LENGTH];
        self.ptr += BANCHO_PACKET_HEADER_LENGTH;

        // Get packet id and payload length
        let PacketHeader { id, payload_length } =
            PacketReader::parse_header(header)?;

        // Read the payload
        let payload = if payload_length == 0 {
            None
        } else {
            let next_payload_length = payload_length as usize;

            self.ptr += next_payload_length;
            self.buf.get(self.ptr - next_payload_length..self.ptr)
        };

        Some(Packet { id, payload })
    }
}

#[derive(Debug, Clone)]
/// [`PacketBuilder`] can help pack bancho packets.
///
/// ### Usages:
/// ```
/// use bancho_packets::{server, PacketBuilder, LoginResult};
///
/// let packet = PacketBuilder::new()
///     .add(server::login_reply(LoginResult::Success(1009)))
///     .add(server::protocol_version(19))
///     .add(server::notification("Welcome to osu!"))
///     .add(server::main_menu_icon(
///         "https://image.png",
///         "https://url.link",
///     ))
///     .add(server::silence_end(0))
///     .add(server::channel_info_end())
///     .build();
/// ```
///
pub struct PacketBuilder {
    buffer: Vec<u8>,
}

impl PacketBuilder {
    #[inline]
    /// Create an empty [`PacketBuilder`].
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    #[inline]
    /// Create [`PacketBuilder`] with capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self { buffer: Vec::with_capacity(capacity) }
    }

    #[inline]
    /// Create [`PacketBuilder`] with [`PacketId`].
    pub fn with_packet_id(packet_id: PacketId) -> Self {
        Self { buffer: new_empty_packet(packet_id) }
    }

    #[inline]
    /// Consume this [`PacketBuilder`] and return data.
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

    #[inline]
    /// Create a [`PacketBuilder`] from multiple packets.
    pub fn from_batch<P, I>(packets: P) -> Self
    where
        I: IntoIterator<Item = u8>,
        P: IntoIterator<Item = I>,
    {
        Self::new().add_batch(packets)
    }

    #[inline]
    /// Batch add packets to [`PacketBuilder`].
    pub fn add_batch<P, I>(mut self, packets: P) -> Self
    where
        I: IntoIterator<Item = u8>,
        P: IntoIterator<Item = I>,
    {
        self.add_batch_ref(packets);
        self
    }

    #[inline]
    /// Batch add packets to [`PacketBuilder`].
    pub fn add_batch_ref<P, I>(&mut self, packets: P) -> &Self
    where
        I: IntoIterator<Item = u8>,
        P: IntoIterator<Item = I>,
    {
        for p in packets {
            self.buffer.extend(p)
        }
        self
    }

    #[inline]
    /// Add an packet to [`PacketBuilder`].
    pub fn add<P>(mut self, packet: P) -> Self
    where
        P: IntoIterator<Item = u8>,
    {
        self.add_ref(packet);
        self
    }

    #[inline]
    /// Add an packet to [`PacketBuilder`].
    pub fn add_ref<P>(&mut self, packet: P) -> &Self
    where
        P: IntoIterator<Item = u8>,
    {
        self.buffer.extend(packet);
        self
    }
}

impl From<Vec<u8>> for PacketBuilder {
    #[inline]
    fn from(buffer: Vec<u8>) -> Self {
        Self { buffer }
    }
}

/// Can use [`PayloadReader`] to read data from type `T` which implements this trait.
pub trait BanchoPacketRead<T> {
    fn read(reader: &mut PayloadReader) -> Option<T>;
}

impl BanchoPacketRead<String> for String {
    #[inline]
    fn read(reader: &mut PayloadReader) -> Option<String> {
        if reader.payload.get(reader.index())? != &0xb {
            return None;
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
    #[inline]
    fn read(reader: &mut PayloadReader) -> Option<bool> {
        Some(reader.read::<i8>()? == 1)
    }
}

macro_rules! impl_number {
    ($($t:ty),+) => {
        $(impl BanchoPacketRead<$t> for $t {
            #[inline]
            fn read(reader: &mut PayloadReader) -> Option<$t> {
                Some(<$t>::from_le_bytes(
                    reader.next_with_length_type::<$t>()?.try_into().ok()?,
                ))
            }
        })+
    };
}

impl_number!(i8, u8, i16, u16, i32, u32, i64, u64);

macro_rules! impl_read_number_array {
    ($($t:ty),+) => {
        $(impl BanchoPacketRead<Vec<$t>> for Vec<$t> {
            #[inline]
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

/// [`BanchoPacketWrite`] is a trait used to convert rust internal data types to bancho packets ([`Vec<u8>`]).
pub trait BanchoPacketWrite {
    /// Convert [`self`] into a bancho packet and write it into `buf` [`Vec<u8>`].
    fn write_buf(self, buf: &mut Vec<u8>);

    #[inline]
    /// Convert [`self`] into a bancho packet [`Vec<u8>`].
    fn as_packet(self) -> Vec<u8>
    where
        Self: Sized,
    {
        let mut buf = Vec::new();
        self.write_buf(&mut buf);
        buf
    }
}

impl BanchoPacketWrite for &str {
    #[inline]
    fn write_buf(self, buf: &mut Vec<u8>) {
        let byte_length = self.len();
        if byte_length > 0 {
            // string length (uleb128)
            let (data, ptr) = u32_to_uleb128(byte_length as u32);
            let length_uleb128 = &data[0..ptr];

            let estimate_len = self.packet_len();
            if buf.capacity() < estimate_len {
                buf.reserve(estimate_len);
            }

            buf.push(0xb);
            buf.extend(length_uleb128);
            buf.extend(self.as_bytes());
        } else {
            buf.push(0);
        }
    }
}

impl BanchoPacketWrite for String {
    #[inline]
    fn write_buf(self, buf: &mut Vec<u8>) {
        let byte_length = self.len();
        if byte_length > 0 {
            // string length (uleb128)
            let (data, ptr) = u32_to_uleb128(byte_length as u32);
            let length_uleb128 = &data[0..ptr];

            let estimate_len = self.packet_len();
            if buf.capacity() < estimate_len {
                buf.reserve(estimate_len);
            }

            buf.push(0xb);
            buf.extend(length_uleb128);
            buf.extend(self.into_bytes());
        } else {
            buf.push(0);
        }
    }
}

impl BanchoPacketWrite for u8 {
    #[inline]
    fn write_buf(self, buf: &mut Vec<u8>) {
        buf.push(self);
    }
}

impl BanchoPacketWrite for &[u8] {
    #[inline]
    fn write_buf(self, buf: &mut Vec<u8>) {
        buf.extend(self);
    }
}

impl BanchoPacketWrite for Vec<u8> {
    #[inline]
    fn write_buf(self, buf: &mut Vec<u8>) {
        buf.extend(self);
    }
}

impl BanchoPacketWrite for bool {
    #[inline]
    fn write_buf(self, buf: &mut Vec<u8>) {
        buf.push(if self { 1 } else { 0 });
    }
}

macro_rules! impl_write_number {
    ($($t:ty),+) => {
        $(
            impl BanchoPacketWrite for $t {
                #[inline]
                fn write_buf(self, buf: &mut Vec<u8>) {
                    buf.extend(self.to_le_bytes())
                }
            }

            impl BanchoPacketLength for $t {
                #[inline]
                fn packet_len(&self) -> usize {
                    std::mem::size_of::<$t>()
                }
            }
        )+
    }
}

impl_write_number!(i8, u16, i16, i32, u32, i64, u64, f32, f64);

macro_rules! impl_write_number_array {
    ($($t:ty),+) => {$(
        impl BanchoPacketWrite for &[$t] { impl_write_number_array!(@bancho_packet_write_inner $t); }
        impl BanchoPacketWrite for Vec<$t> { impl_write_number_array!(@bancho_packet_write_inner $t); }

        impl BanchoPacketLength for &[$t] { impl_write_number_array!(@bancho_packet_length_inner $t); }
        impl BanchoPacketLength for Vec<$t> { impl_write_number_array!(@bancho_packet_length_inner $t); }
    )+};
    (@bancho_packet_write_inner $t:ty) => {
        #[inline]
        fn write_buf(self, buf: &mut Vec<u8>) {
            let estimate_len = self.packet_len();
            if buf.capacity() < estimate_len {
                buf.reserve(estimate_len);
            }

            buf.extend((self.len() as u16).to_le_bytes());
            for int in self {
                buf.extend(int.to_le_bytes())
            }
        }
    };
    (@bancho_packet_length_inner $t:ty) => {
        #[inline]
        fn packet_len(&self) -> usize {
            std::mem::size_of::<u16>() + (std::mem::size_of::<$t>() * self.len())
        }
    }
}

impl_write_number_array!(i8, u16, i16, i32, u32, i64, u64, f32, f64);

impl BanchoPacketWrite for LoginResult {
    #[inline]
    fn write_buf(self, buf: &mut Vec<u8>) {
        match self {
            LoginResult::Success(user_id) => user_id,
            LoginResult::Failed(reason) => reason as i32,
        }
        .write_buf(buf)
    }
}

impl BanchoPacketWrite for MatchUpdate {
    #[inline]
    fn write_buf(mut self, buf: &mut Vec<u8>) {
        let raw_password = std::mem::take(&mut self.password)
            .and_then(|password| {
                if self.send_password {
                    let mut raw = Vec::with_capacity(password.packet_len());
                    password.write_buf(&mut raw);
                    Some(raw)
                } else {
                    Some(EMPTY_STRING_PACKET.to_vec())
                }
            })
            .unwrap_or(b"\x00".to_vec());

        buf.extend(data!(
            self.match_id as u16,
            self.in_progress,
            self.match_type,
            self.play_mods,
            std::mem::take(&mut self.match_name),
            raw_password,
            std::mem::take(&mut self.beatmap_name),
            self.beatmap_id,
            std::mem::take(&mut self.beatmap_md5),
            std::mem::take(&mut self.slot_status),
            std::mem::take(&mut self.slot_teams),
            std::mem::take(&mut self.slot_players),
            self.host_player_id,
            self.match_game_mode,
            self.win_condition,
            self.team_type,
            self.freemods,
            std::mem::take(&mut self.player_mods),
            self.match_seed
        ));
    }
}

impl BanchoPacketWrite for MatchData {
    #[inline]
    fn write_buf(self, buf: &mut Vec<u8>) {
        MatchUpdate { data: self.to_owned(), send_password: true }
            .write_buf(buf);
    }
}

/// [`BanchoPacketLength`] is a trait used to calculate the byte length of the data converted to bancho packet.
pub trait BanchoPacketLength {
    #[inline]
    /// Calculate the byte length of `self` after being converted into a bancho packet,
    /// which is used to allocate [`Vec`] space in advance to improve performance.
    ///
    /// If not implemented, return `0`.
    fn packet_len(&self) -> usize {
        0
    }
}
impl BanchoPacketLength for &str {
    #[inline]
    fn packet_len(&self) -> usize {
        if self.len() > 0 {
            self.as_bytes().len() + 6
        } else {
            1
        }
    }
}

impl BanchoPacketLength for String {
    #[inline]
    fn packet_len(&self) -> usize {
        if self.len() > 0 {
            self.as_bytes().len() + 6
        } else {
            1
        }
    }
}

impl BanchoPacketLength for u8 {
    #[inline]
    fn packet_len(&self) -> usize {
        std::mem::size_of::<u8>()
    }
}

impl BanchoPacketLength for &[u8] {
    #[inline]
    fn packet_len(&self) -> usize {
        self.len()
    }
}

impl BanchoPacketLength for Vec<u8> {
    #[inline]
    fn packet_len(&self) -> usize {
        self.len()
    }
}

impl BanchoPacketLength for bool {
    #[inline]
    fn packet_len(&self) -> usize {
        std::mem::size_of::<bool>()
    }
}

impl BanchoPacketLength for LoginResult {
    #[inline]
    fn packet_len(&self) -> usize {
        std::mem::size_of::<i32>()
    }
}

#[inline]
/// Convert [`u32`] to `uleb128`
pub fn u32_to_uleb128(mut unsigned: u32) -> ([u8; 5], usize) {
    let mut data = [0, 0, 0, 0, 0];
    let mut ptr = 0;

    loop {
        if unsigned < 0x80 {
            break;
        }
        data[ptr] = ((unsigned & 0x7f) | 0x80) as u8;
        ptr += 1;
        unsigned >>= 7;
    }
    data[ptr] = unsigned as u8;

    (data, ptr + 1)
}

#[inline]
/// Convert `uleb128` bytes to [`u32`]
pub fn uleb128_to_u32(uleb128_bytes: &[u8]) -> Option<(u32, usize)> {
    let (mut val, mut shift, mut index) = (0, 0, 0);
    loop {
        let byte = uleb128_bytes.get(index)?;
        index += 1;
        if (byte & 0x80) == 0 {
            val |= (*byte as u32) << shift;
            return Some((val, index));
        }
        val |= ((byte & 0x7f) as u32) << shift;
        shift += 7;
    }
}

#[inline(always)]
/// Initial a packet with PacketId
///
/// Packets posits:
///
/// - 0..=1: [`PacketId`]
/// - 2: `0x0`
/// - 3..=6: packet length ([`i32`] as bytes)
/// - 7..=N: packet data length (`uleb128` as bytes) + data
///
/// Note: The maximum value of [`u8`] is `255`,
/// currently the largest packet id of bancho is `109`, so never overflow.
pub fn new_empty_packet(packet_id: PacketId) -> Vec<u8> {
    vec![packet_id as u8, 0, 0, 0, 0, 0, 0]
}

#[inline(always)]
/// Pack message packet data
pub fn pack_message(
    sender: impl Str,
    content: impl Str,
    target: impl Str,
    sender_id: i32,
) -> Vec<u8> {
    data!(sender, content, target, sender_id)
}

#[inline(always)]
/// Pack channel info packet data
pub fn pack_channel_info(
    name: impl Str,
    title: impl Str,
    player_count: i16,
) -> Vec<u8> {
    data!(name, title, player_count)
}

/// Provide some convenient declarative macros to help build bancho packets.
pub mod macros {
    #[macro_export]
    /// Pack bancho packet data
    ///
    /// ### Usages:
    /// ```
    /// use bancho_packets::*;
    ///
    /// let val_1: i32 = 123;
    /// let val_2: i16 = 50;
    ///
    /// // Single data
    /// data!(val_1);
    ///
    /// // Mutiple data
    /// data!(val_1, val_2);
    ///
    /// // Create packet data with additional capacity
    /// data!(@capacity { 100 }, val_1, val_2);
    /// ```
    macro_rules! data {
        ($($item:expr$(,)*)*) => {
            {
                let mut estimate_capacity = 0;
                $(estimate_capacity += $item.packet_len();)*

                let mut buf = Vec::with_capacity(estimate_capacity);
                $($item.write_buf(&mut buf);)*
                buf
            }
        };
        (@capacity { $capacity:expr }, $($item:expr$(,)*)*) => {
            {
                let mut estimate_capacity = 0;
                $(estimate_capacity += $item.packet_len();)*

                let mut buf = Vec::with_capacity($capacity + estimate_capacity);
                $($item.write_buf(&mut buf);)*
                buf
            }
        }
    }

    #[macro_export]
    /// Pack bancho packets
    ///
    ///
    /// ### Usages:
    /// ```
    /// use bancho_packets::*;
    ///
    /// // Basic
    /// packet!(PacketId::BANCHO_USER_STATS);
    ///
    /// // With data
    /// let data: i32 = 6;
    /// packet!(PacketId::BANCHO_USER_STATS, data);
    ///
    /// // With data
    /// let data = vec![1, 2, 3];
    /// packet!(PacketId::BANCHO_USER_STATS, data);
    ///
    ///
    /// // With complex data
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
        ($packet_id:expr $(,$data:expr)*) => {
            {
                let mut packet = $crate::new_empty_packet($packet_id);
                $($data.write_buf(&mut packet);)*

                let packet_length_bytes =
                    ((packet.len() - $crate::BANCHO_PACKET_HEADER_LENGTH) as i32)
                        .to_le_bytes();

                // `new_empty_packet` always returns a vector of length 7, there will be no null pointer.
                packet[3] = packet_length_bytes[0];
                packet[4] = packet_length_bytes[1];
                packet[5] = packet_length_bytes[2];
                packet[6] = packet_length_bytes[3];

                packet
            }
        }
    }
}
