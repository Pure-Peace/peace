#![allow(dead_code)]
use peace_constants::packets::id;

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

pub trait OsuWrite {
    fn osu_write(&self) -> Vec<u8>;
}

macro_rules! impl_number {
    ($($t:ty),+) => {
        $(impl OsuWrite for $t {
            #[inline(always)]
            fn osu_write(&self) -> Vec<u8> {
                Vec::from(self.to_le_bytes())
            }
        })+
    }
}

impl_number!(u8, u16, i16, i32, u32, i64, f32);

/// Number vec
impl<W: OsuWrite> OsuWrite for &Vec<W> {
    #[inline(always)]
    fn osu_write(&self) -> Vec<u8> {
        let mut ret = Vec::from((self.len() as u16).to_le_bytes());
        for int in self.iter() {
            ret.extend(int.osu_write());
        }
        ret
    }
}

macro_rules! impl_string {
    ($($t:ty),+) => {
        $(impl OsuWrite for $t {
            #[inline(always)]
            fn osu_write(&self) -> Vec<u8> {
                let byte_length = self.len();
                let mut data: Vec<u8> = Vec::with_capacity(byte_length + 3);
                if byte_length > 0 {
                    data.push(11);
                    data.extend(write_uleb128(byte_length as u32));
                    data.extend(self.as_bytes());
                } else {
                    data.push(0);
                }
                data
            }
        })+
    }
}

impl_string!(&String, &str);

impl OsuWrite for bool {
    #[inline(always)]
    fn osu_write(&self) -> Vec<u8> {
        if *self {
            vec![1]
        } else {
            vec![0]
        }
    }
}

#[macro_export]
/// Creating osu!packet data
///
/// # Examples:
/// ```
/// let val_1: i32 = 123;
/// let val_2: i16 = 50;
///
/// // Single data, eq with `val_1.osu_write()`
/// data!(val_1)
///
/// // Mutiple data, default capacity is 30
/// data!(val_1, val_2)
///
/// // Specify initial capacity = 100
/// data!(Cap = 100, val_1, val_2)
/// ```
macro_rules! data {
    ($item:expr) => {
        {
            $item.osu_write()
        }
    };
    ($($item:expr),+) => {
        {
            let mut data = Vec::with_capacity(30);
            $(data.extend($item.osu_write());)+
            data
        }
    };
    (Cap=$capacity:expr;$($item:expr),+) => {
        {
            let mut data = Vec::with_capacity($capacity);
            $(data.extend($item.osu_write());)+
            data
        }
    }
}

#[macro_export]
/// Creating osu!packet
///
/// The first parameter is always packet_id.
///
/// Two cases exist for the remaining parameters:
/// 1. When last parameters ending with a semicolon,
/// it means origin data (impl OsuWrite trait).
/// 2. Otherwise it means packet data.
///
/// # Examples:
/// ```
/// // Origin data here (i32)
/// let data = reply.val();
/// build!(id::BANCHO_USER_STATS, data;)
///
/// // Packet data here (Vec<u8>)
/// let data = reply.val().osu_write();
/// build!(id::BANCHO_USER_STATS, data)
///
/// // Only packet_id
/// build!(id::BANCHO_USER_STATS)
///
/// // Mutiple
/// build!(
///     id::BANCHO_USER_PRESENCE,
///     data!(
///         user_id,
///         username,
///         utc_offset + 24,
///         country_code,
///         (bancho_priv | 0) as u8,
///         longitude,
///         latitude,
///         rank
///     )
/// )
/// ```
macro_rules! build {
    ($packet_id:expr) => {
        {
            let mut packet = vec![$packet_id as u8, 0, 0, 0, 0, 0, 0];
            for (index, value) in ((packet.len() - 7) as i32).to_le_bytes().iter().enumerate() {
                packet[3 + index] = *value;
            }
            packet
        }
    };
    ($packet_id:expr,$($data:expr),*;) => {
        {
            let mut packet = vec![$packet_id as u8, 0, 0, 0, 0, 0, 0];
            $(packet.extend($data.osu_write());)*
            for (index, value) in ((packet.len() - 7) as i32).to_le_bytes().iter().enumerate() {
                packet[3 + index] = *value;
            }
            packet
        }
    };
    ($packet_id:expr,$($data:expr),*) => {
        {
            let mut packet = vec![$packet_id as u8, 0, 0, 0, 0, 0, 0];
            $(packet.extend($data);)*
            for (index, value) in ((packet.len() - 7) as i32).to_le_bytes().iter().enumerate() {
                packet[3 + index] = *value;
            }
            packet
        }
    }
}

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
/// Write message packet
pub fn write_message(sender: &str, sender_id: i32, content: &str, target: &str) -> Vec<u8> {
    data!(sender, content, target, sender_id)
}

#[inline(always)]
pub async fn write_message_async(
    sender: &str,
    sender_id: i32,
    content: &str,
    target: &str,
) -> Vec<u8> {
    data!(sender, content, target, sender_id)
}

#[inline(always)]
pub fn write_channel(name: &str, title: &str, player_count: i16) -> Vec<u8> {
    data!(name, title, player_count)
}

#[inline(always)]
pub fn write_score_frame(
    timestamp: i32,
    id: u8,
    n300: u16,
    n100: u16,
    n50: u16,
    geki: u16,
    katu: u16,
    miss: u16,
    score: i32,
    combo: u16,
    max_combo: u16,
    perfect: bool,
    hp: u8,
    tag_byte: u8,
    score_v2: bool,
) -> Vec<u8> {
    data!(
        timestamp, id, n300, n100, n50, geki, katu, miss, score, combo, max_combo, perfect, hp,
        tag_byte, score_v2
    )
}

#[inline(always)]
pub fn write<W: OsuWrite>(t: W) -> Vec<u8> {
    t.osu_write()
}

#[inline(always)]
pub async fn write_async<W: OsuWrite>(t: W) -> Vec<u8> {
    t.osu_write()
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
