
use anyhow::{anyhow, Error};
use scanner_comms::{self, packets::{Axis, ErrCode, Packet, RotSide}};

pub enum PackType {
    Ok(scanner_comms::packets::packet_ok::OkPacket),
    Err(scanner_comms::packets::packet_err::ErrPacket),
    #[allow(dead_code)]
    Mov(scanner_comms::packets::packet_mov::MovPacket),
    Mes(scanner_comms::packets::packet_mes::MesPacket),
    #[allow(dead_code)]
    Abort(scanner_comms::packets::packet_abort::AbortPacket),
    #[allow(dead_code)]
    Prog(scanner_comms::packets::packet_prog::ProgPacket),
    Fin(scanner_comms::packets::packet_fin::FinPacket),
}

pub fn decode_packet(pack: &mut Vec<u8>) -> Result<PackType, Error> {
    
    match pack[4] {
        1 => {
            let mut out = scanner_comms::packets::packet_ok::OkPacket::new(0, 0, 0);
            let code = scanner_comms::packets::packet_ok::OkPacket::deserialize(pack.as_mut_ptr(), pack.len(), &mut out);
            println!("Ret code: {:?}", code);
            match code {
                0 => Err(anyhow!("Packet mangled")),
                _ => Ok(PackType::Ok(out))
            }
        },
        2 => {
            let mut out = scanner_comms::packets::packet_err::ErrPacket::new(0, ErrCode::UNKNOWN, 0);
            scanner_comms::packets::packet_err::ErrPacket::deserialize(pack.as_mut_ptr(), pack.len(), &mut out);
            Ok(PackType::Err(out))
        },
        3 => {
            let mut out = scanner_comms::packets::packet_mov::MovPacket::new(0, Axis::Horizon, RotSide::Clockwise, 0);
            scanner_comms::packets::packet_mov::MovPacket::deserialize(pack.as_mut_ptr(), pack.len(), &mut out);
            Ok(PackType::Mov(out))
        },
        4 => {
            let mut out = scanner_comms::packets::packet_mes::MesPacket::new(0, 0);
            scanner_comms::packets::packet_mes::MesPacket::deserialize(pack.as_mut_ptr(), pack.len(), &mut out);
            Ok(PackType::Mes(out))
        },
        5 => {
            let mut out = scanner_comms::packets::packet_abort::AbortPacket::new(0);
            scanner_comms::packets::packet_abort::AbortPacket::deserialize(pack.as_mut_ptr(), pack.len(), &mut out);
            Ok(PackType::Abort(out))
        },
        6 => {
            let mut out = scanner_comms::packets::packet_prog::ProgPacket::new(0, 0, 0);
            scanner_comms::packets::packet_prog::ProgPacket::deserialize(pack.as_mut_ptr(), pack.len(), &mut out);
            Ok(PackType::Prog(out))
        },
        7 => {
            let mut out = scanner_comms::packets::packet_fin::FinPacket::new(0, 0);
            scanner_comms::packets::packet_fin::FinPacket::deserialize(pack.as_mut_ptr(), pack.len(), &mut out);
            Ok(PackType::Fin(out))
        },
        _ => { Err(Error::msg("Packet unknown!")) }
    }
    
}