use crate::{data, io::traits::writing::OsuWrite, packets::structures::PacketId};

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
/// ```rust,ignore
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
pub fn new_packet(packet_id: PacketId) -> Vec<u8> {
    vec![packet_id as u8, 0, 0, 0, 0, 0, 0]
}

#[inline(always)]
/// Simple packaging for output(new(packet_id))
///
/// !Note: Packet length is included
pub fn simple_pack(packet_id: PacketId) -> Vec<u8> {
    output(new_packet(packet_id))
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
pub fn osu_write<W>(t: W) -> Vec<u8>
where
    W: OsuWrite,
{
    t.osu_write()
}
