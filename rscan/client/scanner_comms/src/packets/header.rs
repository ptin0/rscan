use super::PacketType;
use super::ErrCode;
use super::CRC_CALC;

use byteorder::ByteOrder;

/// Header struct that contains packet metadata
/// 
/// len - length of the packet incluging the header
/// packet_id - the id number to identify the particular packets
/// packet_type - 8-bit enum determining packet type
/// crc - 16-bit crc value for packet integryty verification.
#[repr(C)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct Header {
    pub len: u8,
    pub packet_id: u16,
    pub packet_type: PacketType,
    pub crc: u16,
}

impl Header {
    /// Facotry method for building the Header
    /// 
    /// packet_length - 8-bit length of the packet including the header
    /// packet_id - 16-bit packet id to identify the particular packet
    /// packet_type - 8-bit enum determining packet type
    pub fn new(pcket_length: u8, packet_id: u16, packet_type: PacketType) -> Self {
        Header {
            len: pcket_length,
            packet_id,
            packet_type,
            crc: 0,
        }
    }
    
    /// This method serializes Header into provided slice of at least header's size
    /// out - a target slice
    /// 
    /// @ret usize - length that has been written
    pub fn serialize(&self, out: &mut [u8]) -> usize {
        out[0] = self.len;
        byteorder::NetworkEndian::write_u16(&mut out[1..3], self.packet_id);
        out[3] = self.packet_type as u8;
        byteorder::NetworkEndian::write_u16(&mut out[4..6], self.crc);

        6
    }

    /// This method deserializes Header from the provided input slice
    /// 
    /// input - slice containing serialized packet
    /// 
    /// @ret Result<(), ErrCode> - Returns the error enum derserialization fails
    pub fn deserialize(&mut self, input: &[u8]) -> Result<(), ErrCode> {
        let len = input[0];
        let packet_type = input[3];
        let packet_id = byteorder::NetworkEndian::read_u16(&input[1..3]);
        let crc = byteorder::NetworkEndian::read_u16(&input[4..6]);
        
        match packet_type {
            0x00 => return Err(ErrCode::BROKEN),
            0x01 => self.packet_type = PacketType::Ok,
            0x02 => self.packet_type = PacketType::Err,
            0x03 => self.packet_type = PacketType::Mov,
            0x04 => self.packet_type = PacketType::Mes,
            0x05 => self.packet_type = PacketType::Abord,
            0x06 => self.packet_type = PacketType::Prog,
            0x07 => self.packet_type = PacketType::Fin,
            _ => return Err(ErrCode::BROKEN),
        }

        self.len = len;
        self.packet_id = packet_id;
        //self.packet_type = packet_type;
        self.crc = crc;

        Ok(())
    }

    /// Destructive method!!!
    pub fn validate_crc(&self, buf: &mut [u8]) -> bool {
        buf[4] = 0;
        buf[5] = 0;
        
        self.crc == CRC_CALC.checksum(&buf[..self.len as usize])
    }

    #[cfg(test)]
    pub fn zero_crc(&mut self) {
        self.crc = 0;
    }

    pub const fn size_of() -> usize { 6 } // Remember to update max serialization size!!!
}
