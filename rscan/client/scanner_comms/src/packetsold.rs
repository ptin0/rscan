use byteorder::ByteOrder;

use crate::ErrCode;

const CRC_CALC: crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_XMODEM);

/// Enum type encoding packet types.
/// 
/// OK - acknowledgement that the command/mewasure has been received correctly
/// ERR - acknowledgement that the command/mewasure has been received incorrectly or the state of the device prevents the execution of a command
/// MOV - move motors by given offsets
/// MES - measurement data
/// ABORT - abort the current scan
/// PROG - contains scan parameters
/// FIN - scan has been finished
/// UNKNOWN - packet type is not known, something gone wrong. DO NOT SEND THIS VALUE!!!
/// 
#[repr(C)]
#[derive(Clone, Copy)]
#[allow(dead_code)] //To temporarly disable misleading warnings!
pub enum PacketType {
    Ok      = 0x00,
    Err     = 0x01,
    Mov     = 0x02,
    Mes     = 0x03,
    Abord   = 0x04,
    Prog    = 0x05,
    Fin     = 0x06,
    Uknown = 0xff,
}
#[warn(dead_code)]


pub trait Packet {
    extern "C" fn serialize(&self, out: *mut u8, out_length: usize) -> usize;

    extern "C" fn deserialize(input: *mut u8, in_length: usize, out: &mut OkPacket) -> usize;
}

#[repr(C)]
struct Header {
    len: u8,
    packet_id: u16,
    packet_type: PacketType,
    crc: u16,
}

impl Header {
    pub fn new(pcket_length: u8, packet_id: u16, packet_type: PacketType) -> Self {
        Header {
            len: pcket_length,
            packet_id,
            packet_type,
            crc: 0,
        }
    }
    
    pub fn serialize(&self, out: &mut [u8]) -> usize {
        out[0] = self.len;
        out[1..3].clone_from_slice(&self.packet_id.to_ne_bytes());
        out[3] = self.packet_type as u8;
        out[4..6].clone_from_slice(&self.crc.to_ne_bytes());

        6
    }

    pub fn deserialize(input: &[u8]) -> Result<Self, ErrCode> {
        let len = input[0];
        let packet_type = input[3];
        let packet_id = byteorder::NetworkEndian::read_u16(&input[1..3]);
        let crc = byteorder::NetworkEndian::read_u16(&input[4..6]);

        if packet_type != PacketType::Ok as u8 { return Err(ErrCode::BROKEN); }
        
        let packet_type = PacketType::Ok;

        Ok(Header {
            len,
            packet_id,
            packet_type,
            crc,
        })
    }

    pub const fn size_of() -> usize { 6 } // Remember to update max serialization size!!!
}

#[repr(C)]
pub struct OkPacket {
    header: Header,
    sentinel: u8,
}

impl OkPacket {
    #[export_name = "ok_packet_new"]
    pub extern "C" fn new(packet_id: u16) -> Self {
        let size = core::mem::size_of::<Header>() as u8;
        let header = Header::new(size, packet_id, PacketType::Ok);

        Self {
            header,
            sentinel: 0xa0,
        }
    }

    const fn size_of() -> usize { Header::size_of() + 1 } // Remember to update max serialization size!!!
}

impl Packet for OkPacket {
    #[export_name = "ok_packet_serialize"]
    extern "C" fn serialize(&self, out: *mut u8, out_length: usize) -> usize {

        if out_length < OkPacket::size_of() + 2 { return 0; }

        // Trapping a raw pointer into usable output slice
        let out = unsafe { core::slice::from_raw_parts_mut(out, out_length) };

        // Initializing temporary buffer where the packet gets constructed
        let mut tmp_buf: [u8; OkPacket::size_of()] = [0xff; OkPacket::size_of()];

        // Serializing the initiali header. Crc is zero!!!
        let header_len = self.header.serialize(&mut tmp_buf);

        // Start of payload serialization
        tmp_buf[header_len] = self.sentinel;
        // End of payload serialization

        // Crc calculatrion step
        let crc = &CRC_CALC.checksum(&tmp_buf);

        // Swapping initiali zero in Crc for an actual Crc value
        tmp_buf[header_len - 2..header_len].copy_from_slice(&crc.to_ne_bytes());

        // Adding COBS framing and sending to the output provided by the caller
        corncobs::encode_buf(&tmp_buf, out)
    }

    extern "C" fn deserialize(input_ptr: *mut u8, in_length: usize, out: &mut OkPacket) -> usize {
        if in_length != OkPacket::size_of() + 2 {return 0;}

        let input = unsafe { core::slice::from_raw_parts(input_ptr, in_length) };

        let mut tmp_buf: [u8; OkPacket::size_of() + 2] = [0; OkPacket::size_of() + 2];

        let len = corncobs::decode_buf(input, &mut tmp_buf).unwrap_or(0);
        if len == 0 { return 0; }

        let mut crc_slice = &tmp_buf[Header::size_of() - 2 .. Header::size_of()];

        out.header.crc = u16::from_ne_bytes(crc_slice.try_into().unwrap());

        crc_slice.copy_from_slice(&[0, 0]);

        let rx_crc = CRC_CALC.checksum(&tmp_buf[0..OkPacket::size_of()]);

        if rx_crc != out.header.crc { return -1; }

        // TODO: Move header extraction to header deserialization!!!

    }
    
}
