use crate::packets::{utils, PacketId};

pub struct PacketBuilder {
    content: Vec<u8>,
}

impl PacketBuilder {
    #[inline]
    /// Initial an empty packet
    pub fn new() -> Self {
        PacketBuilder {
            content: utils::empty(),
        }
    }

    #[inline]
    /// Initial a packet with id
    ///
    /// !Note: Packet length is not included,
    pub fn with(packet_id: PacketId) -> Self {
        PacketBuilder {
            content: utils::new_packet(packet_id),
        }
    }

    #[inline]
    /// Initial from packet data
    pub fn from(packet: Vec<u8>) -> PacketBuilder {
        PacketBuilder { content: packet }
    }

    #[inline]
    pub fn from_multiple(packets: &mut [Vec<u8>]) -> PacketBuilder {
        let mut packet = utils::empty();
        for i in packets.iter_mut() {
            packet.append(i)
        }
        PacketBuilder { content: packet }
    }

    #[inline]
    pub fn merge(packets: &mut [Vec<u8>]) -> Vec<u8> {
        let mut packet = utils::empty();
        for i in packets.iter_mut() {
            packet.append(i)
        }
        packet
    }

    #[inline]
    pub fn add_multiple_ref(&mut self, packets: &mut [Vec<u8>]) {
        for i in packets.iter_mut() {
            self.content.append(i)
        }
    }

    #[inline]
    pub fn add_multiple(mut self, packets: &mut [Vec<u8>]) -> PacketBuilder {
        for i in packets.iter_mut() {
            self.content.append(i)
        }
        self
    }

    #[inline]
    /// Add packet data
    pub fn add(mut self, packet: Vec<u8>) -> PacketBuilder {
        self.content.extend(packet);
        self
    }

    #[inline]
    /// Add packet data
    pub fn add_ref(&mut self, packet: Vec<u8>) -> &PacketBuilder {
        self.content.extend(packet);
        self
    }

    #[inline]
    /// Write out the packet
    pub fn write_out(self) -> Vec<u8> {
        self.content
    }

    #[inline]
    /// Pack the packet
    ///
    /// !Note: Packet length will be added
    pub fn pack(self) -> Vec<u8> {
        utils::output(self.content)
    }
}

pub mod macros {
    #[macro_export]
    /// Creating osu!packet data
    ///
    /// # Examples:
    /// ```
    /// use bancho_packets::{data, traits::writing::*};
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
            let mut buf = Vec::with_capacity(30);
            $item.osu_write(&mut buf);
            buf
        }
    };
    ($($item:expr),+) => {
        {
            let mut buf = Vec::with_capacity(30);
            $($item.osu_write(&mut buf);)+
            buf
        }
    };
    ({ $capacity:expr }; $($item:expr),+) => {
        {
            let mut buf = Vec::with_capacity($capacity);
            $($item.osu_write(&mut buf);)+
            buf
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
    /// use bancho_packets::{PacketId, build, data, out_packet, traits::writing::*};
    ///
    /// // Origin data here (i32)
    /// let data: i32 = 6;
    /// build!(PacketId::BANCHO_USER_STATS, data);
    ///
    /// // Packet data here (Vec<u8>)
    /// let data = vec![1, 2, 3];
    /// build!(PacketId::BANCHO_USER_STATS, data);
    ///
    /// // Only packet_id
    /// build!(PacketId::BANCHO_USER_STATS);
    ///
    /// // Complex
    /// let user_id: i32 = 1000;
    /// let username: &str = "PurePeace";
    ///
    /// build!(
    ///     PacketId::BANCHO_USER_PRESENCE,
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
            $($data.osu_write(&mut p);)*
            out_packet!(p)
        }
    }
}
}
