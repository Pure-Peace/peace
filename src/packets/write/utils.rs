#![allow(dead_code)]
pub struct PacketBuilder {
    content: Vec<u8>,
}

impl PacketBuilder {
    /// Initial an empty packet
    pub fn new() -> Self {
        PacketBuilder { content: empty() }
    }

    /// Initial a packet with id and length
    pub fn with(packet_id: u8) -> Self {
        PacketBuilder { content: new(packet_id) }
    }

    /// Initial from packet data
    pub fn from(packet: Vec<u8>) -> PacketBuilder {
        PacketBuilder { content: packet }
    }

    pub fn from_multiple(packets: &[Vec<u8>]) -> PacketBuilder {
        let mut packet = empty();
        for i in packets.iter() {
            packet.extend(i)
        }
        PacketBuilder { content: packet }
    }

    /// Add packet data
    pub fn add(mut self, packet: Vec<u8>) -> PacketBuilder {
        self.content.extend(packet);
        self
    }

    /// Build packet
    pub fn done(self) -> Vec<u8> {
        self.content
    }

    /// Write out packet
    pub fn write_out(self) -> Vec<u8> {
        output(self.content)
    }
}

pub trait Integer {
    fn to_bytes(&self) -> Vec<u8>;
}

impl Integer for i32 {
    fn to_bytes(&self) -> Vec<u8> {
        Vec::from(self.to_le_bytes())
    }
}

/// Create a empty packets
pub fn empty() -> Vec<u8> {
    Vec::with_capacity(11)
}

/// Initial a packet by id
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
pub fn new(packet_id: u8) -> Vec<u8> {
    vec![packet_id, 0, 0, 0, 0, 0, 0]
}

/// Add packet length and write out
pub fn output(mut packet: Vec<u8>) -> Vec<u8> {
    for (index, value) in ((packet.len() - 7) as i32).to_le_bytes().iter().enumerate() {
        packet[3 + index] = *value;
    }
    packet
}

/// Write string packet
pub fn write_string(string: &str) -> Vec<u8> {
    let byte_length = string.len();
    let mut data: Vec<u8> = Vec::with_capacity(byte_length + 3);
    if byte_length > 0 {
        data.push(11); // 0x0b, means not empty
        data.extend(uleb128(byte_length as u32));
        data.extend(string.as_bytes());
    } else {
        data.push(0); // 0x00, means empty
    }
    data
}

/// Write message packet
/// 
/// ### impl 1:
/// ```
/// PacketBuilder::from_multiple(&[
///     write_string(sender),
///     write_string(content),
///     write_string(channel),
///     write_integer(sender_id),
/// ])
/// .done()
/// ```
/// 
/// ### impl 2:
/// ```
/// PacketBuilder::new()
///     .add(write_string(sender))
///     .add(write_string(content))
///     .add(write_string(channel))
///     .add(write_integer(sender_id))
///     .done()
/// ```
/// 
/// ### impl 3 (best performance):
/// ```
/// now impl
/// ```
pub fn write_message(sender: &str, sender_id: i32, content: &str, channel: &str) -> Vec<u8> {
    let mut data: Vec<u8> = Vec::with_capacity(30);
    data.extend(write_string(sender));
    data.extend(write_string(content));
    data.extend(write_string(channel));
    data.extend(write_integer(sender_id));
    data
    
}

/// Write integer packet
pub fn write_integer<T: Integer>(integer: T) -> Vec<u8> {
    integer.to_bytes()
}

/// Unsigned to uleb128
fn uleb128(mut unsigned: u32) -> Vec<u8> {
    let mut data: Vec<u8> = Vec::with_capacity(2);
    while unsigned >= 0x80 {
        data.push(((unsigned & 0x7f) | 0x80) as u8);
        unsigned >>= 7;
    }
    data.push(unsigned as u8);
    data
}
