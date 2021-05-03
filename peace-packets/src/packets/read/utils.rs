use peace_constants::packets::id;
use std::convert::TryInto;
use std::str;

use num_traits::FromPrimitive;

pub trait ReadInteger<T> {
    fn from_le_bytes(data: &[u8]) -> Option<T>;
    fn from_be_bytes(data: &[u8]) -> Option<T>;
    fn as_usize(self) -> usize;
}

macro_rules! impl_read_integer {
    ($($t:ty),+) => {
        $(impl ReadInteger<$t> for $t {
            #[inline(always)]
            fn from_le_bytes(data: &[u8]) -> Option<$t> {
                Some(<$t>::from_le_bytes(match data.try_into() {
                    Ok(d) => d,
                    Err(_) => {
                        return None;
                    }
                }))
            }
            #[inline(always)]
            fn from_be_bytes(data: &[u8]) -> Option<$t> {
                Some(<$t>::from_be_bytes(match data.try_into() {
                    Ok(d) => d,
                    Err(_) => {
                        return None;
                    }
                }))
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
    pub fn read_integer<Integer: ReadInteger<Integer>>(&mut self) -> Option<Integer> {
        let data = self.read(std::mem::size_of::<Integer>())?;
        Integer::from_le_bytes(data)
    }

    #[inline(always)]
    pub fn read_i32_list<IntLength: ReadInteger<IntLength>>(&mut self) -> Option<Vec<i32>> {
        let length_data = self.read(std::mem::size_of::<IntLength>())?;
        let int_count = IntLength::from_le_bytes(length_data)?.as_usize();

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
    pub fn read_message(&mut self) -> Option<Message> {
        Some(Message {
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

        Some(
            match str::from_utf8(data) {
                Ok(s) => s,
                Err(_) => return None,
            }
            .to_string(),
        )
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
    pub current_packet: id,
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
    pub fn next(&mut self) -> Option<(id, Option<&[u8]>)> {
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
            println!("[PacketReader]: unknown packet id({})", header[0]);
            id::OSU_UNKNOWN_PACKET
        });
        let length = u32::from_le_bytes(match header[3..=6].try_into() {
            Ok(b) => b,
            Err(_) => return Some((packet_id, None)),
        });

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
                self.buf.get(self.index - self.payload_length..self.index)
            }
        };

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
            u32::from_le_bytes(match header[3..=6].try_into() {
                Ok(b) => b,
                Err(_) => {
                    return None;
                }
            }),
        ))
    }
}

#[inline(always)]
pub fn read_uleb128(slice: &[u8]) -> Option<(u32, usize)> {
    let (mut val, mut shift, mut index) = (0, 0, 0);
    loop {
        let byte = slice.get(index)?;
        index += 1;
        if (byte & 0x80) == 0 {
            val |= (*byte as u32) << shift;
            return Some((val, index));
        }
        val |= ((byte & 0x7f) as u32) << shift;
        shift += 7;
    }
}
