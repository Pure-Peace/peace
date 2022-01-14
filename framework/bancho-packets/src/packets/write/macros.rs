#[macro_export]
/// Creating osu!packet data
///
/// # Examples:
/// ```
/// use bancho_packets::{data, write_traits::*};
/// 
/// let val_1: i32 = 123;
/// let val_2: i16 = 50;
///
/// // Single data, eq with `val_1.osu_write()`
/// data!(val_1);
///
/// // Mutiple data, default capacity is 30
/// data!(val_1, val_2);
///
/// // Specify initial capacity = 100
/// data!({ 100 }; val_1, val_2);
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
    ({ $capacity:expr }; $($item:expr),+) => {
        {
            let mut data = Vec::with_capacity($capacity);
            $(data.extend($item.osu_write());)+
            data
        }
    }
}

#[macro_export]
macro_rules! out_packet {
    ($packet:expr) => {{
        for (index, value) in (($packet.len() - 7) as i32)
            .to_le_bytes()
            .iter()
            .enumerate()
        {
            $packet[3 + index] = *value;
        }
        $packet
    }};
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
/// use bancho_packets::{id, build, data, out_packet, write_traits::*};
/// 
/// // Origin data here (i32)
/// let data: i32 = 6;
/// build!(id::BANCHO_USER_STATS, data);
///
/// // Packet data here (Vec<u8>)
/// let data = vec![1, 2, 3];
/// build!(id::BANCHO_USER_STATS, data);
///
/// // Only packet_id
/// build!(id::BANCHO_USER_STATS);
///
/// // Complex
/// let user_id: i32 = 1000;
/// let username: &str = "PurePeace";
/// 
/// build!(
///     id::BANCHO_USER_PRESENCE,
///     data!(
///         user_id,
///         username
///     )
/// );
/// ```
macro_rules! build {
    ($packet_id:expr) => {
        {
            let mut p = vec![$packet_id as u8, 0, 0, 0, 0, 0, 0];
            out_packet!(p)
        }
    };
    ($packet_id:expr,$($data:expr),*) => {
        {
            let mut p = vec![$packet_id as u8, 0, 0, 0, 0, 0, 0];
            $(p.extend($data.osu_write());)*
            out_packet!(p)
        }
    }
}
