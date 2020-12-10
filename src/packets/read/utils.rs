use std::convert::TryInto;

use actix_web::web::{Bytes, Data};
use async_std::sync::RwLock;

use num_traits::{FromPrimitive, ToPrimitive};

use crate::{
    constants::id,
    database::Database,
    objects::{Player, PlayerData, PlayerSessions},
    packets,
    types::ChannelList,
};

pub struct HandlerData<'a> {
    pub player_sessions: &'a Data<RwLock<PlayerSessions>>,
    pub database: &'a Data<Database>,
    pub channel_list: &'a Data<RwLock<ChannelList>>,
    pub token: &'a String,
    pub player_data: PlayerData,
}

#[derive(Debug)]
pub struct ClientPacket {}

impl ClientPacket {
    pub async fn handle<'a>(&self, handler_data: &HandlerData<'a>) {
        println!("{:?} {:?}", self, handler_data.player_data);
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
    pub fn from_bytes(body: Bytes) -> Self {
        PacketReader {
            buf: body.to_vec(),
            index: 0,
            current_packet: id::OSU_UNKNOWN_PACKET,
            payload_length: 0,
            finish: false,
            payload_count: 0,
            packet_count: 0,
        }
    }

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

    pub fn payload(&self) -> Option<Vec<u8>> {
        match self.payload_length {
            0 => None,
            _ => Some(self.buf[self.index..self.index + self.payload_length].to_vec()),
        }
    }

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
    /// Read packet header: (type, length)
    pub fn next(&mut self) -> Option<(id, Option<&[u8]>)> {
        if (self.buf.len() - self.index) < 7 {
            self.finish = true;
            return None;
        }
        // Slice header data [u8; 7]
        let header = &self.buf[self.index..self.index + 7];
        self.index += 7;

        // Get packet id and length
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

        // Convert packet id to enum and return
        Some((packet_id, payload))
    }

    #[inline(always)]
    /// Read packet header: (type, length)
    pub fn read_header(body: Vec<u8>) -> Option<(id, u32)> {
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
