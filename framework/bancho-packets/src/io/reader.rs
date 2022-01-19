use num_traits::FromPrimitive;
use std::convert::TryInto;
use std::str;

use crate::{
    io::{traits::reading::NumberAsBytes, utils::read_uleb128},
    packets::{BanchoMessage, Packet, PacketHeader, PacketId},
};

pub struct PayloadReader<'a> {
    pub payload: &'a [u8],
    pub index: usize,
}

impl<'a> PayloadReader<'a> {
    #[inline(always)]
    pub fn new(payload: &'a [u8]) -> Self {
        PayloadReader { payload, index: 0 }
    }

    #[inline(always)]
    fn read(&mut self, length: usize) -> Option<&[u8]> {
        let data = self.payload.get(self.index..self.index + length)?;
        self.index += length;
        Some(data)
    }

    #[inline(always)]
    pub fn read_integer<N: NumberAsBytes<N>>(&mut self) -> Option<N> {
        let data = self.read(std::mem::size_of::<N>())?;
        N::from_le_bytes(data)
    }

    #[inline(always)]
    pub fn read_i32_list<N: NumberAsBytes<N>>(&mut self) -> Option<Vec<i32>> {
        let length_data = self.read(std::mem::size_of::<N>())?;
        let int_count = N::from_le_bytes(length_data)?.as_usize();

        let mut data: Vec<i32> = Vec::with_capacity(int_count);
        for _ in 0..int_count {
            data.push(i32::from_le_bytes(match self.read(4)?.try_into() {
                Ok(u) => u,
                Err(_) => return None,
            }));
        }
        Some(data)
    }

    #[inline(always)]
    pub fn read_message(&mut self) -> Option<BanchoMessage> {
        Some(BanchoMessage {
            sender: self.read_string()?,
            content: self.read_string()?,
            target: self.read_string()?,
            sender_id: self.read_integer()?,
        })
    }

    #[inline(always)]
    pub fn read_string(&mut self) -> Option<String> {
        if self.payload.get(self.index)? != &11u8 {
            return None;
        }
        self.index += 1;
        let data_length = self.read_uleb128()? as usize;

        let cur = self.index;
        self.index += data_length;
        let data = self.payload.get(cur..self.index)?;

        Some(str::from_utf8(data).ok()?.into())
    }

    #[inline(always)]
    pub fn read_uleb128(&mut self) -> Option<u32> {
        let (val, length) = read_uleb128(&self.payload.get(self.index..)?)?;
        self.index += length;
        Some(val)
    }
}

pub struct PacketReader {
    pub buf: Vec<u8>,
    pub index: usize,
    pub payload_length: usize,
    pub finish: bool,
    pub payload_count: u16,
    pub packet_count: u16,
}

impl PacketReader {
    #[inline(always)]
    pub fn from_vec(body: Vec<u8>) -> Self {
        PacketReader {
            buf: body,
            index: 0,
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
        self.payload_length = 0;
        self.payload_count = 0;
        self.packet_count = 0;
    }

    #[inline(always)]
    /// Read the osu!client packet: (packet id, payload)
    pub fn next(&mut self) -> Option<Packet> {
        if (self.buf.len() - self.index) < 7 {
            self.finish = true;
            return None;
        }
        // Slice packet header data [u8; 7],
        // including packet id, payload length
        let header = &self.buf[self.index..self.index + 7];
        self.index += 7;

        // Get packet id and payload length
        let PacketHeader { id, payload_length } = PacketReader::read_header(header)?;
        self.packet_count += 1;

        // Read the payload
        let payload = if payload_length == 0 {
            None
        } else {
            self.payload_count += 1;
            self.payload_length = payload_length as usize;
            // Skip this payload at next call
            self.index += self.payload_length;
            self.buf.get(self.index - self.payload_length..self.index)
        };

        Some(Packet { id, payload })
    }

    #[inline(always)]
    /// Read packet header: (type, length)
    pub fn read_header(header: &[u8]) -> Option<PacketHeader> {
        let id = *header.get(0)?;
        let packet_id = PacketId::from_u8(id).unwrap_or_else(|| {
            println!("[PacketReader]: unknown packet id({})", id);
            PacketId::OSU_UNKNOWN_PACKET
        });
        let payload_length = u32::from_le_bytes(header[3..=6].try_into().ok()?);
        Some(PacketHeader {
            id: packet_id,
            payload_length,
        })
    }
}
