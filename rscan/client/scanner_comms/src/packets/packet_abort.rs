//use std::println;

use byteorder::ByteOrder;

use super::header::Header;
use super::PacketType;
use super::Packet;
use super::CRC_CALC;




#[repr(C)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct AbortPacket {
    header: Header,
    sentinel: u8,
}

impl AbortPacket {
    #[no_mangle]
    #[export_name = "abord_packet_new"]
    pub extern "C" fn new(packet_id: u16) -> Self {
        let size = AbortPacket::size_of() as u8;
        let header = Header::new(size, packet_id, PacketType::Abord);

        Self {
            header,
            sentinel: 0x0a
        }
    }

    const fn size_of() -> usize { Header::size_of() + 1 } // Remember to update max serialization size!!!
}

impl Packet for AbortPacket {
    #[no_mangle]
    #[export_name = "abort_packet_serialize"]
    extern "C" fn serialize(&self, out: *mut u8, out_length: usize) -> usize {
        if out_length < AbortPacket::size_of() + 2 { return 0; }

        // Trapping a raw pointer into usable output slice
        let out = unsafe { core::slice::from_raw_parts_mut(out, out_length) };

        // Initializing temporary buffer where the packet gets constructed
        let mut tmp_buf: [u8; AbortPacket::size_of()] = [0xff; AbortPacket::size_of()];

        // Serializing the initial header. Crc is zero!!!
        let header_len = self.header.serialize(&mut tmp_buf);
        tmp_buf[header_len] = self.sentinel;

        // Crc calculatrion step
        let crc = CRC_CALC.checksum(&tmp_buf);

        // Swapping initiali zero in Crc for an actual Crc value
        byteorder::NetworkEndian::write_u16(&mut tmp_buf[header_len - 2..header_len], crc);
        
        // Adding COBS framing and sending to the output provided by the caller
        corncobs::encode_buf(&tmp_buf, out)
    }

    #[no_mangle]
    #[export_name = "abort_packet_deserialize"]
    extern "C" fn deserialize(input_ptr: *mut u8, in_length: usize, out: &mut AbortPacket) -> usize {
        // Checking if provided packet has a vaid size
        if in_length != AbortPacket::size_of() + 2 {return 0;}

        // Trapping raw pointer in a useful slice
        let input = unsafe { core::slice::from_raw_parts(input_ptr, in_length) };

        // Initialization of temporary buffer for deserizaliztion
        let mut tmp_buf: [u8; AbortPacket::size_of() + 2] = [0; AbortPacket::size_of() + 2];

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
                out.sentinel = tmp_buf[6];

                // Test code required for assertion
                #[cfg(test)] {
                    out.header.zero_crc();
                }

                // Return packet length (not counting farming)
                return 7;
            }
        }
    }
    
}