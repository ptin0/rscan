use byteorder::ByteOrder;

use super::header::Header;
use super::PacketType;
use super::Packet;
use super::CRC_CALC;




#[repr(C)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct FinPacket {
    header: Header,
    pub number_of_points: u16,
}

impl FinPacket {
    #[no_mangle]
    #[export_name = "fin_packet_new"]
    pub extern "C" fn new(packet_id: u16, number_of_points: u16) -> Self {
        let size = FinPacket::size_of() as u8;
        let header = Header::new(size, packet_id, PacketType::Fin);

        Self {
            header,
            number_of_points,
        }
    }

    const fn size_of() -> usize { Header::size_of() + 2 } // Remember to update max serialization size!!!
}

impl Packet for FinPacket {
    #[no_mangle]
    #[export_name = "fin_packet_serialize"]
    extern "C" fn serialize(&self, out: *mut u8, out_length: usize) -> usize {
        if out_length < FinPacket::size_of() + 2 { return 0; }

        // Trapping a raw pointer into usable output slice
        let out = unsafe { core::slice::from_raw_parts_mut(out, out_length) };

        // Initializing temporary buffer where the packet gets constructed
        let mut tmp_buf: [u8; FinPacket::size_of()] = [0xff; FinPacket::size_of()];

        // Serializing the initial header. Crc is zero!!!
        let header_len = self.header.serialize(&mut tmp_buf);

        // Start of payload serialization
        byteorder::NetworkEndian::write_u16(&mut tmp_buf[header_len..header_len+2], self.number_of_points);
        // End of payload serialization

        // Crc calculatrion step
        let crc = CRC_CALC.checksum(&tmp_buf);

        // Swapping initiali zero in Crc for an actual Crc value
        byteorder::NetworkEndian::write_u16(&mut tmp_buf[header_len - 2..header_len], crc);

        // Adding COBS framing and sending to the output provided by the caller
        corncobs::encode_buf(&tmp_buf, out)
    }

    #[no_mangle]
    #[export_name = "fin_packet_deserialize"]
    extern "C" fn deserialize(input_ptr: *mut u8, in_length: usize, out: &mut FinPacket) -> usize {
        if in_length != FinPacket::size_of() + 2 {return 0;}

        // Trapping raw pointer in a useful slice
        let input = unsafe { core::slice::from_raw_parts(input_ptr, in_length) };

        // Initialization of temporary buffer for deserizaliztion
        let mut tmp_buf: [u8; FinPacket::size_of() + 2] = [0; FinPacket::size_of() + 2];

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
                out.number_of_points = byteorder::NetworkEndian::read_u16(&tmp_buf[6..8]);

                // Test code required for assertion
                #[cfg(test)] {
                    out.header.zero_crc();
                }

                // Return packet length (not counting farming)
                return FinPacket::size_of();
            }
        }
    }
    
}