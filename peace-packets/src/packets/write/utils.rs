#![allow(dead_code)]
use peace_constants::id;

pub struct PacketBuilder {
    content: Vec<u8>,
}

impl PacketBuilder {
    #[inline(always)]
    /// Initial an empty packet
    pub fn new() -> Self {
        PacketBuilder { content: empty() }
    }

    #[inline(always)]
    /// Initial a packet with id
    ///
    /// !Note: Packet length is not included,
    pub fn with(packet_id: id) -> Self {
        PacketBuilder {
            content: new(packet_id),
        }
    }

    #[inline(always)]
    /// Initial from packet data
    pub fn from(packet: Vec<u8>) -> PacketBuilder {
        PacketBuilder { content: packet }
    }

    #[inline(always)]
    pub fn from_multiple(packets: &mut [Vec<u8>]) -> PacketBuilder {
        let mut packet = empty();
        for i in packets.iter_mut() {
            packet.append(i)
        }
        PacketBuilder { content: packet }
    }

    #[inline(always)]
    pub fn merge(packets: &mut [Vec<u8>]) -> Vec<u8> {
        let mut packet = empty();
        for i in packets.iter_mut() {
            packet.append(i)
        }
        packet
    }

    #[inline(always)]
    pub fn add_multiple_ref(&mut self, packets: &mut [Vec<u8>]) {
        for i in packets.iter_mut() {
            self.content.append(i)
        }
    }

    #[inline(always)]
    pub fn add_multiple(mut self, packets: &mut [Vec<u8>]) -> PacketBuilder {
        for i in packets.iter_mut() {
            self.content.append(i)
        }
        self
    }

    #[inline(always)]
    /// Add packet data
    pub fn add(mut self, packet: Vec<u8>) -> PacketBuilder {
        self.content.extend(packet);
        self
    }

    #[inline(always)]
    /// Add packet data
    pub fn add_ref(&mut self, packet: Vec<u8>) -> &PacketBuilder {
        self.content.extend(packet);
        self
    }

    #[inline(always)]
    /// Write out the packet
    pub fn write_out(self) -> Vec<u8> {
        self.content
    }

    #[inline(always)]
    /// Pack the packet
    ///
    /// !Note: Packet length will be added
    pub fn pack(self) -> Vec<u8> {
        output(self.content)
    }
}

pub trait Number {
    fn to_bytes(&self) -> Vec<u8>;
}

macro_rules! impl_number {
    ($($t:ty),+) => {
        $(impl Number for $t {
            #[inline(always)]
            fn to_bytes(&self) -> Vec<u8> {
                Vec::from(self.to_le_bytes())
            }
        })+
    }
}

impl_number!(u8, i16, i32, u32, i64, f32);

#[inline(always)]
/// Create a empty packets
pub fn empty() -> Vec<u8> {
    Vec::with_capacity(11)
}

#[inline(always)]
/// Initial a packet by id
///
/// !Note: Packet length is not included,
///
/// !Requires output() to add packet length.
///
///
/// Packets posit:
/// ```
/// [0..=1]: packet id
/// [2]: null
/// [3..=6]: packet length
/// [7..=N]: data length(uleb128) + data
/// ```
/// The maximum value of u8 is 255,
///
/// but currently the largest packet id of bancho is only 109,
///
/// so I think it is sufficient to insert the packet_id in the first position
///
pub fn new(packet_id: id) -> Vec<u8> {
    vec![packet_id as u8, 0, 0, 0, 0, 0, 0]
}

#[inline(always)]
/// Simple packaging for output(new(packet_id))
///
/// !Note: Packet length is included
pub fn simple_pack(packet_id: id) -> Vec<u8> {
    output(new(packet_id))
}

#[inline(always)]
/// Add packet length and write out
pub fn output(mut packet: Vec<u8>) -> Vec<u8> {
    for (index, value) in ((packet.len() - 7) as i32).to_le_bytes().iter().enumerate() {
        packet[3 + index] = *value;
    }
    packet
}

#[inline(always)]
/// Write string packet
pub fn write_string(string: &str) -> Vec<u8> {
    let byte_length = string.len();
    let mut data: Vec<u8> = Vec::with_capacity(byte_length + 3);
    if byte_length > 0 {
        data.push(11); // 0x0b, means not empty
        data.extend(write_uleb128(byte_length as u32));
        data.extend(string.as_bytes());
    } else {
        data.push(0); // 0x00, means empty
    }
    data
}

#[inline(always)]
/// Write string packet
pub fn write_string_async(string: &str) -> Vec<u8> {
    let byte_length = string.len();
    let mut data: Vec<u8> = Vec::with_capacity(byte_length + 3);
    if byte_length > 0 {
        data.push(11); // 0x0b, means not empty
        data.extend(write_uleb128(byte_length as u32));
        data.extend(string.as_bytes());
    } else {
        data.push(0); // 0x00, means empty
    }
    data
}

#[inline(always)]
/// Write message packet
///
/// ### impl 1:
/// ```
/// PacketBuilder::from_multiple(&[
///     write_string(sender),
///     write_string(content),
///     write_string(channel_name),
///     write_num(sender_id),
/// ])
/// .done()
/// ```
///
/// ### impl 2:
/// ```
/// PacketBuilder::new()
///     .add(write_string(sender))
///     .add(write_string(content))
///     .add(write_string(channel_name))
///     .add(write_num(sender_id))
///     .done()
/// ```
///
/// ### impl 3 (best performance):
/// ```
/// now impl
/// ```
pub fn write_message(sender: &str, sender_id: i32, content: &str, target: &str) -> Vec<u8> {
    let mut data: Vec<u8> = Vec::with_capacity(30);
    data.extend(write_string(sender));
    data.extend(write_string_async(content));
    data.extend(write_string(target));
    data.extend(write_num(sender_id));
    data
}

#[inline(always)]
pub fn write_message_sync(sender: &str, sender_id: i32, content: &str, target: &str) -> Vec<u8> {
    let mut data: Vec<u8> = Vec::with_capacity(30);
    data.extend(write_string(sender));
    data.extend(write_string(content));
    data.extend(write_string(target));
    data.extend(write_num(sender_id));
    data
}

#[inline(always)]
/// Write integer packet
pub fn write_num<N: Number>(num: N) -> Vec<u8> {
    num.to_bytes()
}

#[inline(always)]
pub fn write_channel(name: &str, title: &str, player_count: i16) -> Vec<u8> {
    let mut data: Vec<u8> = Vec::with_capacity(30);
    data.extend(write_string(name));
    data.extend(write_string(title));
    data.extend(write_num(player_count));
    data
}

#[inline(always)]
/// Write int list packet
pub fn write_int_list<N: Number>(integer_list: &Vec<N>) -> Vec<u8> {
    let mut ret = Vec::from((integer_list.len() as u16).to_le_bytes());
    for int in integer_list {
        ret.extend(int.to_bytes());
    }
    ret
}

#[inline(always)]
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
