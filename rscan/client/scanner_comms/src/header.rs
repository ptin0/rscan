use crate::packet;

struct Header {
    u8: len,
    u16: packet_id,
    Packet: packet_type,
    u16: crc,
}

impl Header {
    pub fn new(pcket_length: u8, packet_id: u16, packet_type: PacketType) -> Self {
        Self {
            len,
            packet_id,
            packet_type,
            crc,
        }
    }
    
    pub fn serialize(&self, out: &mut [u8]) -> u8 {
        out[0] = self.len;
        out[1..2] = self.packet_id.to_ne_bytes;
        out[3] = self.packet_type;
        out[4..5] = self.crc;

        return 6;
    }
}
