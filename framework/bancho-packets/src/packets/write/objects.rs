use super::utils;
use crate::id;

pub struct PacketBuilder {
    content: Vec<u8>,
}

impl PacketBuilder {
    #[inline(always)]
    /// Initial an empty packet
    pub fn new() -> Self {
        PacketBuilder {
            content: utils::empty(),
        }
    }

    #[inline(always)]
    /// Initial a packet with id
    ///
    /// !Note: Packet length is not included,
    pub fn with(packet_id: id) -> Self {
        PacketBuilder {
            content: utils::new(packet_id),
        }
    }

    #[inline(always)]
    /// Initial from packet data
    pub fn from(packet: Vec<u8>) -> PacketBuilder {
        PacketBuilder { content: packet }
    }

    #[inline(always)]
    pub fn from_multiple(packets: &mut [Vec<u8>]) -> PacketBuilder {
        let mut packet = utils::empty();
        for i in packets.iter_mut() {
            packet.append(i)
        }
        PacketBuilder { content: packet }
    }

    #[inline(always)]
    pub fn merge(packets: &mut [Vec<u8>]) -> Vec<u8> {
        let mut packet = utils::empty();
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
        utils::output(self.content)
    }
}
