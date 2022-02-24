use num_traits::FromPrimitive;
use std::convert::TryInto;

use crate::{
    packets::{Packet, PacketHeader, PacketId},
    read_uleb128,
    traits::reading::OsuRead,
};

pub struct PayloadReader<'a> {
    pub(crate) payload: &'a [u8],
    pub(crate) index: usize,
}

impl<'a> PayloadReader<'a> {
    #[inline(always)]
    pub fn new(payload: &'a [u8]) -> Self {
        PayloadReader { payload, index: 0 }
    }

    #[inline(always)]
    pub fn index(&self) -> usize {
        self.index
    }

    #[inline(always)]
    pub fn payload(&self) -> &'a [u8] {
        self.payload
    }

    #[inline(always)]
    pub fn add_index(&mut self, offset: usize) -> usize {
        self.index += offset;
        self.index
    }

    #[inline(always)]
    pub fn sub_index(&mut self, offset: usize) -> usize {
        self.index -= offset;
        self.index
    }

    #[inline(always)]
    pub(crate) fn next(&mut self, length: usize) -> Option<&[u8]> {
        self.index += length;
        Some(self.payload.get(self.index - length..self.index)?)
    }

    #[inline(always)]
    pub fn read<T>(&mut self) -> Option<T>
    where
        T: OsuRead<T>,
    {
        T::read(self)
    }

    #[inline(always)]
    pub(crate) fn read_uleb128(&mut self) -> Option<u32> {
        let (val, length) = read_uleb128(&self.payload.get(self.index..)?)?;
        self.index += length;
        Some(val)
    }
}

pub struct PacketReader<'a> {
    buf: &'a [u8],
    index: usize,
    payload_length: usize,
    finish: bool,
}

impl<'a> PacketReader<'a> {
    #[inline(always)]
    pub fn new(buf: &'a [u8]) -> Self {
        PacketReader {
            buf,
            index: 0,
            payload_length: 0,
            finish: false,
        }
    }

    #[inline(always)]
    pub fn index(&self) -> usize {
        self.index
    }

    #[inline(always)]
    pub fn buf(&self) -> &'a [u8] {
        self.buf
    }

    #[inline(always)]
    pub fn payload_len(&self) -> usize {
        self.payload_length
    }

    #[inline(always)]
    pub fn is_finished(&self) -> bool {
        self.finish
    }

    #[inline(always)]
    // Reset the packet reader
    pub fn reset(&mut self) {
        self.finish = false;
        self.index = 0;
        self.payload_length = 0;
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

    #[inline(always)]
    /// Read packet header: (type, length)
    pub fn read_header(header: &[u8]) -> Option<PacketHeader> {
        let id = *header.get(0)?;
        Some(PacketHeader {
            id: PacketId::from_u8(id).unwrap_or(PacketId::OSU_UNKNOWN_PACKET),
            payload_length: u32::from_le_bytes(header[3..=6].try_into().ok()?),
        })
    }
}
