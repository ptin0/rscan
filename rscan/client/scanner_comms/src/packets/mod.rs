use crc::Table;

mod header;
pub mod packet_ok;
pub mod packet_err;
pub mod packet_mov;
pub mod packet_mes;
pub mod packet_abort;
pub mod packet_prog;
pub mod packet_fin;

const CRC_CALC: crc::Crc<u16, Table<1>> = crc::Crc::<u16, Table<1>>::new(&crc::CRC_16_XMODEM);

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
#[cfg_attr(test, derive(PartialEq, Debug))]
#[allow(dead_code)] //To temporarly disable misleading warnings!
pub enum PacketType {
    Ok      = 0x01,
    Err     = 0x02,
    Mov     = 0x03,
    Mes     = 0x04,
    Abord   = 0x05,
    Prog    = 0x06,
    Fin     = 0x07,
    Uknown = 0xff,
}

#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(test, derive(PartialEq, Debug))]//To temporarly disable misleading warnings!
pub enum Axis {
    Horizon = 0x00,
    Azimuth = 0x01,
}

#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(test, derive(PartialEq, Debug))]//T
pub enum RotSide {
    Clockwise = 0x00,
    CounterClockwise = 0x01,
}


/// Enum type encoding error codes of ERR packets.
/// 
/// UNKNOWN - somemthing gone very bad. The sender cannot sedcribe what exactly.
/// BUSY - the deivce is currently executing other task preventing execution of the command.
/// BROKEN - the received packet is broken, plese retransmit.
///
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(test, derive(PartialEq, Debug))]
#[allow(dead_code)]
pub enum ErrCode {
    UNKNOWN,
    BUSY,
    BROKEN,
}

pub trait Packet {
    extern "C" fn serialize(&self, out: *mut u8, out_length: usize) -> usize;

    extern "C" fn deserialize(input: *mut u8, in_length: usize, out: &mut Self) -> usize;
}

