use crate::{data, packets::PacketId, traits::writing::OsuWrite};

pub struct PacketBuilder {
    content: Vec<u8>,
}

impl PacketBuilder {
    #[inline]
    /// Initial an empty packet
    pub fn new() -> Self {
        Self {
            content: Vec::with_capacity(16),
        }
    }

    #[inline]
    /// Create a builder with PacketId
    pub fn with(packet_id: PacketId) -> Self {
        Self {
            content: Self::new_packet(packet_id),
        }
    }

    #[inline]
    /// Initial from packet data
    pub fn from(packet: Vec<u8>) -> Self {
        Self { content: packet }
    }

    #[inline]
    pub fn from_multiple(packets: &[&[u8]]) -> Self {
        Self::new().add_multiple(packets)
    }

    #[inline]
    pub fn pack(packets: &[&[u8]]) -> Vec<u8> {
        Self::new().add_multiple(packets).write_out()
    }

    #[inline]
    pub fn add_multiple(mut self, packets: &[&[u8]]) -> Self {
        self.add_multiple_ref(packets);
        self
    }

    #[inline]
    pub fn add_multiple_ref(&mut self, packets: &[&[u8]]) -> &Self {
        for &i in packets {
            self.content.extend(i)
        }
        self
    }

    #[inline]
    /// Add packet data
    pub fn add(mut self, packet: &[u8]) -> Self {
        self.add_ref(packet);
        self
    }

    #[inline]
    /// Add packet data
    pub fn add_ref(&mut self, packet: &[u8]) -> &Self {
        self.content.extend(packet);
        self
    }

    #[inline]
    /// Write out the packet
    pub fn write_out(self) -> Vec<u8> {
        self.content
    }

    #[inline]
    /// Build the packet and add length
    pub fn build(mut self) -> Vec<u8> {
        Self::add_osu_length(&mut self.content);
        self.write_out()
    }

    #[inline]
    pub fn content(&self) -> &Vec<u8> {
        &self.content
    }

    #[inline]
    /// Initial a packet with PacketId
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
        vec![packet_id.val(), 0, 0, 0, 0, 0, 0]
    }

    #[inline]
    /// Build a simple packet
    pub fn simple_build(packet_id: PacketId) -> Vec<u8> {
        let mut pk = Self::new_packet(packet_id);
        Self::add_osu_length(&mut pk);
        pk
    }

    #[inline]
    /// Add osu packet length
    pub fn add_osu_length(packet: &mut Vec<u8>) {
        for (index, value) in ((packet.len() - 7) as i32).to_le_bytes().iter().enumerate() {
            packet[3 + index] = *value;
        }
    }

    #[inline]
    /// Write message packet
    pub fn write_message(sender: &str, sender_id: i32, content: &str, target: &str) -> Vec<u8> {
        data!(sender, content, target, sender_id)
    }

    #[inline]
    pub fn write_channel(name: &str, title: &str, player_count: i16) -> Vec<u8> {
        data!(name, title, player_count)
    }

    #[inline]
    pub fn osu_write<W>(data: W) -> Vec<u8>
    where
        W: OsuWrite,
    {
        let mut buf = Vec::new();
        data.osu_write(&mut buf);
        buf
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
    /// use bancho_packets::prelude::*;
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
