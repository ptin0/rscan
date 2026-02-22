#![no_std]
#![no_main]
// Panic handling tactics:
#[cfg(debug_assertions)]
use panic_semihosting as _;


#[cfg(all(target_arch = "arm", target_os = "none", not(debug_assertions)))]
use panic_abort as _;
//use panic_semihosting as _;

pub mod packets;

//#[no_mangle]
//pub extern "C" fn aaa(
//    a: PacketType,
//    b: ErrCode,
//    c: Envelope
//) {
//
//}

#[cfg(test)]
mod tests {

    use crate::packets::{Axis, RotSide};

    use super::*;
    use super::packets::Packet;

    #[test]
    fn ok_serial_deserial() {

        let mut buf: [u8; 20] = [0; 20];
        let buf_ptr = buf.as_mut_ptr();

        let test_ok = packets::packet_ok::OkPacket::new(123, 0xa0, 0x0a);

        let len = test_ok.serialize(buf_ptr, 20);

        let mut rx_packet = packets::packet_ok::OkPacket::new(0, 0, 0);

        let _len = packets::packet_ok::OkPacket::deserialize(buf_ptr, len, &mut rx_packet);

        assert_eq!(test_ok, rx_packet);
    }

    #[test]
    fn abort_serial_deserial() {

        let mut buf: [u8; 20] = [0; 20];
        let buf_ptr = buf.as_mut_ptr();

        let test_ok = packets::packet_abort::AbortPacket::new(123);

        let len = test_ok.serialize(buf_ptr, 20);

        let mut rx_packet = packets::packet_abort::AbortPacket::new(0);

        let _len = packets::packet_abort::AbortPacket::deserialize(buf_ptr, len, &mut rx_packet);

        assert_eq!(test_ok, rx_packet);
    }

    #[test]
    fn err_serial_deserial() {

        let mut buf: [u8; 20] = [0; 20];
        let buf_ptr = buf.as_mut_ptr();

        let test_ok = packets::packet_err::ErrPacket::new(123, packets::ErrCode::BROKEN, 42);

        let len = test_ok.serialize(buf_ptr, 20);

        let mut rx_packet = packets::packet_err::ErrPacket::new(0, packets::ErrCode::UNKNOWN, 0);

        let _len = packets::packet_err::ErrPacket::deserialize(buf_ptr, len, &mut rx_packet);

        assert_eq!(test_ok, rx_packet);
    }

    #[test]
    fn fin_serial_deserial() {

        let mut buf: [u8; 20] = [0; 20];
        let buf_ptr = buf.as_mut_ptr();

        let test_ok = packets::packet_fin::FinPacket::new(123, 15123);

        let len = test_ok.serialize(buf_ptr, 20);

        let mut rx_packet = packets::packet_fin::FinPacket::new(0, 0);

        let _len = packets::packet_fin::FinPacket::deserialize(buf_ptr, len, &mut rx_packet);

        assert_eq!(test_ok, rx_packet);
    }
    
    #[test]
    fn prog_serial_deserial() {

        let mut buf: [u8; 20] = [0; 20];
        let buf_ptr = buf.as_mut_ptr();

        let test_ok = packets::packet_prog::ProgPacket::new(123, 42, 11);

        let len = test_ok.serialize(buf_ptr, 20);

        let mut rx_packet = packets::packet_prog::ProgPacket::new(0, 0, 0);

        let _len = packets::packet_prog::ProgPacket::deserialize(buf_ptr, len, &mut rx_packet);

        assert_eq!(test_ok, rx_packet);
    }
    
    #[test]
    fn mes_serial_deserial() {

        let mut buf: [u8; 20] = [0; 20];
        let buf_ptr = buf.as_mut_ptr();

        let test_ok = packets::packet_mes::MesPacket::new(123, 67890);

        let len = test_ok.serialize(buf_ptr, 20);

        let mut rx_packet = packets::packet_mes::MesPacket::new(0, 0);

        let _len = packets::packet_mes::MesPacket::deserialize(buf_ptr, len, &mut rx_packet);

        assert_eq!(test_ok, rx_packet);
    }
    
    #[test]
    fn mov_serial_deserial() {

        let mut buf: [u8; 20] = [0; 20];
        let buf_ptr = buf.as_mut_ptr();

        let test_ok = packets::packet_mov::MovPacket::new(123, Axis::Azimuth, RotSide::Clockwise, 32);

        let len = test_ok.serialize(buf_ptr, 20);

        let mut rx_packet = packets::packet_mov::MovPacket::new(0, Axis::Horizon, RotSide::CounterClockwise, 0);

        let _len = packets::packet_mov::MovPacket::deserialize(buf_ptr, len, &mut rx_packet);

        assert_eq!(test_ok, rx_packet);
    }
    
}
