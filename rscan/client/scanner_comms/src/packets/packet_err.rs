use byteorder::ByteOrder;

use super::header::Header;
use super::PacketType;
use super::Packet;
use super::CRC_CALC;
use super::ErrCode;




#[repr(C)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct ErrPacket {
    pub header: Header,
    pub error: ErrCode,
    pub packet_id: u16,
}

impl ErrPacket {
    #[no_mangle]
    #[export_name = "err_packet_new"]
    pub extern "C" fn new(packet_id: u16, err_code: ErrCode, responding_to_id: u16) -> Self {
        let size = ErrPacket::size_of() as u8;
        let header = Header::new(size, packet_id, PacketType::Err);

        Self {
            header,
            error: err_code,
            packet_id: responding_to_id,
        }
    }

    const fn size_of() -> usize { Header::size_of() + 3 } // Remember to update max serialization size!!!
}

impl Packet for ErrPacket {
    #[no_mangle]
    #[export_name = "err_packet_serialize"]
    extern "C" fn serialize(&self, out: *mut u8, out_length: usize) -> usize {
        if out_length < ErrPacket::size_of() + 2 { return 0; }

        // Trapping a raw pointer into usable output slice
        let out = unsafe { core::slice::from_raw_parts_mut(out, out_length) };

        // Initializing temporary buffer where the packet gets constructed
        let mut tmp_buf: [u8; ErrPacket::size_of()] = [0xff; ErrPacket::size_of()];

        // Serializing the initial header. Crc is zero!!!
        let header_len = self.header.serialize(&mut tmp_buf);

        // Start of payload serialization
        tmp_buf[header_len] = self.error as u8;
        byteorder::NetworkEndian::write_u16(&mut tmp_buf[header_len+1..header_len+3], self.packet_id);
        // End of payload serialization

        // Crc calculatrion step
        let crc = CRC_CALC.checksum(&tmp_buf);

        // Swapping initiali zero in Crc for an actual Crc value
        byteorder::NetworkEndian::write_u16(&mut tmp_buf[header_len - 2..header_len], crc);

        // Adding COBS framing and sending to the output provided by the caller
        corncobs::encode_buf(&tmp_buf, out)
    }

    #[no_mangle]
    #[export_name = "err_packet_deserialize"]
    extern "C" fn deserialize(input_ptr: *mut u8, in_length: usize, out: &mut ErrPacket) -> usize {
        if in_length != ErrPacket::size_of() + 2 {return 0;}

        // Trapping raw pointer in a useful slice
        let input = unsafe { core::slice::from_raw_parts(input_ptr, in_length) };

        // Initialization of temporary buffer for deserizaliztion
        let mut tmp_buf: [u8; ErrPacket::size_of() + 2] = [0; ErrPacket::size_of() + 2];

        // Removing COBS framing
        let len = corncobs::decode_buf(input, &mut tmp_buf).unwrap_or(0);
        // Return 0 if COBS removal failed
        if len == 0 { return 0; }

        // Read header struct
        match out.header.deserialize(&tmp_buf) {
            Err(_) => return 0,
            Ok(_) => {
                // Validate Crc
                if !out.header.validate_crc(&mut tmp_buf) { return 0; }

                // Deserialize sentinel
                match tmp_buf[6] {
                    0x00 => out.error = ErrCode::UNKNOWN,
                    0x01 => out.error = ErrCode::BUSY,
                    0x02 => out.error = ErrCode::BROKEN,
                    _ => return 0,
                }
                out.packet_id = byteorder::NetworkEndian::read_u16(& tmp_buf[7..9]);

                // Test code required for assertion
                #[cfg(test)] {
                    out.header.zero_crc();
                }

                // Return packet length (not counting farming)
                return ErrPacket::size_of();
            }
        }
    }
    
}
