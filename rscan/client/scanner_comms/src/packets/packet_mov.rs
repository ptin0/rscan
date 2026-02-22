use byteorder::ByteOrder;

use super::header::Header;
use super::Axis;
use super::PacketType;
use super::Packet;
use super::RotSide;
use super::CRC_CALC;




#[repr(C)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct MovPacket {
    header: Header,
    pub axis: Axis,
    pub side: RotSide,
    pub steps: u8,
}

impl MovPacket {
    #[no_mangle]
    #[export_name = "mov_packet_new"]
    pub extern "C" fn new(packet_id: u16, axis: Axis, side: RotSide, steps: u8) -> Self {
        let size = MovPacket::size_of() as u8;
        let header = Header::new(size, packet_id, PacketType::Mov);

        Self {
            header,
            axis,
            side,
            steps,
        }
    }

    const fn size_of() -> usize { Header::size_of() + 3 } // Remember to update max serialization size!!!
}

impl Packet for MovPacket {
    #[no_mangle]
    #[export_name = "mov_packet_serialize"]
    extern "C" fn serialize(&self, out: *mut u8, out_length: usize) -> usize {
        if out_length < MovPacket::size_of() + 2 { return 0; }

        // Trapping a raw pointer into usable output slice
        let out = unsafe { core::slice::from_raw_parts_mut(out, out_length) };

        // Initializing temporary buffer where the packet gets constructed
        let mut tmp_buf: [u8; MovPacket::size_of()] = [0xff; MovPacket::size_of()];

        // Serializing the initial header. Crc is zero!!!
        let header_len = self.header.serialize(&mut tmp_buf);

        // Start of payload serialization
        tmp_buf[header_len] = self.axis as u8;
        tmp_buf[header_len+1] = self.side as u8;
        tmp_buf[header_len+2] = self.steps;
        // End of payload serialization

        // Crc calculatrion step
        let crc = CRC_CALC.checksum(&tmp_buf);

        // Swapping initiali zero in Crc for an actual Crc value
        byteorder::NetworkEndian::write_u16(&mut tmp_buf[header_len - 2..header_len], crc);

        // Adding COBS framing and sending to the output provided by the caller
        corncobs::encode_buf(&tmp_buf, out)
    }

    #[no_mangle]
    #[export_name = "mov_packet_deserialize"]
    extern "C" fn deserialize(input_ptr: *mut u8, in_length: usize, out: &mut MovPacket) -> usize {
        if in_length != MovPacket::size_of() + 2 {return 0;}

        // Trapping raw pointer in a useful slice
        let input = unsafe { core::slice::from_raw_parts(input_ptr, in_length) };

        // Initialization of temporary buffer for deserizaliztion
        let mut tmp_buf: [u8; MovPacket::size_of() + 2] = [0; MovPacket::size_of() + 2];

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
                    0x00 => out.axis = Axis::Horizon,
                    0x01 => out.axis = Axis::Azimuth,
                    _ => return 0,
                }
                
                match tmp_buf[7] {
                    0x00 => out.side = RotSide::Clockwise,
                    0x01 => out.side = RotSide::CounterClockwise,
                    _ => return 0,
                }
                
                out.steps = tmp_buf[8];

                // Test code required for assertion
                #[cfg(test)] {
                    out.header.zero_crc();
                }

                // Return packet length (not counting farming)
                return MovPacket::size_of();
            }
        }
    }
    
}