use std::str;
use std::{convert::TryInto, sync::Weak};

use actix_web::web::{Bytes, Data};
use async_std::sync::RwLock;

use num_traits::FromPrimitive;

use crate::{
    constants::id,
    database::Database,
    objects::{Player, PlayerData, PlayerSessions},
    settings::bancho::BanchoConfig,
    types::ChannelList,
};

pub struct HandlerContext<'a> {
    pub request_ip: &'a String,
    pub token: &'a String,
    pub id: i32,
    pub name: &'a String,
    pub data: &'a PlayerData,
    pub weak_player: &'a Weak<RwLock<Player>>,
    pub player_sessions: &'a Data<RwLock<PlayerSessions>>,
    pub database: &'a Data<Database>,
    pub channel_list: &'a Data<RwLock<ChannelList>>,
    pub bancho_config: &'a Data<RwLock<BanchoConfig>>,
    pub payload: &'a [u8],
}

pub trait ReadInteger<T> {
    fn from_le_bytes(data: &[u8]) -> T;
    fn from_be_bytes(data: &[u8]) -> T;
    fn as_usize(self) -> usize;
}

macro_rules! impl_read_integer {
    ($($t:ty),+) => {
        $(impl ReadInteger<$t> for $t {
            #[inline(always)]
            fn from_le_bytes(data: &[u8]) -> $t {
                <$t>::from_le_bytes(data.try_into().unwrap())
            }
            #[inline(always)]
            fn from_be_bytes(data: &[u8]) -> $t {
                <$t>::from_be_bytes(data.try_into().unwrap())
            }
            #[inline(always)]
            fn as_usize(self) -> usize {
                self as usize
            }
        })+
    }
}

impl_read_integer!(i8, u8, i16, u16, i32, u32, i64, u64);

#[derive(Debug)]
pub struct Message {
    pub sender: String,
    pub content: String,
    pub target: String,
    pub sender_id: i32,
}

pub struct PayloadReader<'a> {
    pub payload: &'a [u8],
    pub index: usize,
}

impl<'a> PayloadReader<'a> {
    pub fn new(payload: &'a [u8]) -> Self {
        PayloadReader { payload, index: 0 }
    }

    #[inline(always)]
    fn read(&mut self, length: usize) -> &[u8] {
        let data = &self.payload[self.index..self.index + length];
        self.index += length;
        data
    }

    #[inline(always)]
    pub async fn read_integer<Integer: ReadInteger<Integer>>(&mut self) -> Integer {
        let data = self.read(std::mem::size_of::<Integer>());
        Integer::from_le_bytes(data)
    }

    #[inline(always)]
    pub async fn read_i32_list<IntLength: ReadInteger<IntLength>>(&mut self) -> Vec<i32> {
        let length_data = self.read(std::mem::size_of::<IntLength>());
        let int_count = IntLength::from_le_bytes(length_data).as_usize();

        let mut data: Vec<i32> = Vec::with_capacity(int_count);
        for _ in 0..int_count {
            data.push(i32::from_le_bytes(self.read(4).try_into().unwrap()));
        }
        data
    }

    #[inline(always)]
    pub async fn read_message(&mut self) -> Message {
        Message {
            sender: self.read_string().await,
            content: self.read_string().await,
            target: self.read_string().await,
            sender_id: self.read_integer().await,
        }
    }

    #[inline(always)]
    pub async fn read_string(&mut self) -> String {
        if self.payload[self.index] != 11 {
            return String::new();
        }
        self.index += 1;
        let data_length = self.read_uleb128() as usize;

        let cur = self.index;
        self.index += data_length;
        let data = &self.payload[cur..self.index];

        str::from_utf8(data).unwrap_or("").to_string()
    }

    #[inline(always)]
    pub fn read_uleb128(&mut self) -> u32 {
        let (val, length) = read_uleb128(&self.payload[self.index..]);
        self.index += length;
        val
    }
}

pub struct PacketReader {
    pub buf: Vec<u8>,
    pub index: usize,
    pub current_packet: id,
    pub payload_length: usize,
    pub finish: bool,
    pub payload_count: u16,
    pub packet_count: u16,
}

impl PacketReader {
    #[inline(always)]
    pub fn from_bytes(body: Bytes) -> Self {
        PacketReader::from_vec(body.to_vec())
    }

    #[inline(always)]
    pub fn from_vec(body: Vec<u8>) -> Self {
        PacketReader {
            buf: body,
            index: 0,
            current_packet: id::OSU_UNKNOWN_PACKET,
            payload_length: 0,
            finish: false,
            payload_count: 0,
            packet_count: 0,
        }
    }

    #[inline(always)]
    // Reset the packet reader
    pub fn reset(&mut self) {
        self.finish = false;
        self.index = 0;
        self.current_packet = id::OSU_UNKNOWN_PACKET;
        self.payload_length = 0;
        self.payload_count = 0;
        self.packet_count = 0;
    }

    #[inline(always)]
    /// Read the osu!client packet: (packet id, payload)
    pub async fn next(&mut self) -> Option<(id, Option<&[u8]>)> {
        if (self.buf.len() - self.index) < 7 {
            self.finish = true;
            return None;
        }
        // Slice packet header data [u8; 7],
        // including packet id, payload length
        let header = &self.buf[self.index..self.index + 7];
        self.index += 7;

        // Get packet id and payload length
        let packet_id = id::from_u8(header[0]).unwrap_or_else(|| {
            warn!("PacketReader: unknown packet id({})", header[0]);
            id::OSU_UNKNOWN_PACKET
        });
        let length = u32::from_le_bytes(header[3..=6].try_into().unwrap());

        self.packet_count += 1;
        self.current_packet = packet_id.clone();

        // Read the payload
        let payload = match length {
            0 => None,
            _ => {
                self.payload_count += 1;
                self.payload_length = length as usize;
                // Skip this payload at next call
                self.index += self.payload_length;
                Some(&self.buf[self.index - self.payload_length..self.index])
            }
        };

        Some((packet_id, payload))
    }

    #[inline(always)]
    /// Read packet header: (type, length)
    pub async fn read_header(body: Vec<u8>) -> Option<(id, u32)> {
        if body.len() < 7 {
            return None;
        }
        let header = &body[..7];
        Some((
            id::from_u8(header[0]).unwrap_or(id::OSU_UNKNOWN_PACKET),
            u32::from_le_bytes(header[3..=6].try_into().unwrap()),
        ))
    }
}

#[inline(always)]
pub fn read_uleb128(slice: &[u8]) -> (u32, usize) {
    let (mut val, mut shift, mut index) = (0, 0, 0);
    loop {
        let byte = slice[index];
        index += 1;
        if (byte & 0x80) == 0 {
            val |= (byte as u32) << shift;
            return (val, index);
        }
        val |= ((byte & 0x7f) as u32) << shift;
        shift += 7;
    }
}
