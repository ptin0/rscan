use std::{io::Write, sync::{Arc, Mutex}};
#[allow(unused_imports)]
use log::{debug, error, info, warn};
use scanner_comms::packets::packet_fin::FinPacket;
use tokio::sync::mpsc::Sender;
use scanner_comms::packets::Packet;

use crate::state::GeneralState;

type CState = Arc<Mutex<super::state::ClientState>>;

pub fn ok_pack(state: CState, pack: scanner_comms::packets::packet_ok::OkPacket) {
    let mut state = state.lock().unwrap();
    match state.ack {
        super::state::AckState::Normal => warn!("Got unexpected ok for packet!"),
        super::state::AckState::Awaiting => {
            match state.general {
                GeneralState::Programming => {
                    let start_line = pack.sentinel;
                    let start_point = pack.sentinel2;
                    state.out_file.write_all(&[start_line, start_point]).unwrap();
                    state.general = super::state::GeneralState::Measure;
                    state.ack = super::state::AckState::Normal;
                },
                _ => { },
            }
            info!("Previous packet ok received!");
            state.ack = super::state::AckState::Normal;
            state.consec_error_counter = 0;
        }
    }
}

pub fn err_handle(state: CState, pack: scanner_comms::packets::packet_err::ErrPacket, send_chan: Sender<Vec<u8>>) {
    let mut state = state.lock().unwrap();
    match state.ack {
        super::state::AckState::Normal => warn!("Got unexpected error for packet!"),
        _ => (),
    }
    match pack.error {
        scanner_comms::packets::ErrCode::BROKEN => {
            state.consec_error_counter += 1;
            if state.consec_error_counter >= 6 {
                error!("Device replied with error BROKEN after 5 attempts! Loop has been broken! State is unknown, you continue on your own responsibility, here there be dragons!");
                return ;
            }
            warn!("Packet reported broken: {:?}", state.last_pack);
            tokio::task::block_in_place(|| {
                let handle = tokio::runtime::Handle::current();
                handle.block_on(send_chan.send(state.last_pack.clone())).unwrap();
            });
            warn!("Retransmitting...")
        }
        
        scanner_comms::packets::ErrCode::UNKNOWN => {
            error!("Received unknow error from the target!");
        }
        
        scanner_comms::packets::ErrCode::BUSY => warn!("Target busy, belay command until target expects it."),
    };
}

pub fn mes_handle(state: CState, pack: scanner_comms::packets::packet_mes::MesPacket, send_chan: Sender<Vec<u8>>, progress_chan: Sender<u16>) {
    let mut state = state.lock().unwrap();
    match state.general {
        GeneralState::Measure => {
            
            state.out_file.write_all(&pack.mes.to_be_bytes()).unwrap();
            
            let stp = state.make_step() - 1;
            
            tokio::task::block_in_place(|| {
                let handle = tokio::runtime::Handle::current();
                handle.block_on(progress_chan.send(stp)).unwrap();
            });

            info!("Wrote {:?} to file", pack.mes);
            let resp = scanner_comms::packets::packet_ok::OkPacket::new(123, 0xa0, 0x0a);
            
            let mut pack = Vec::with_capacity(10);
        
            let size = resp.serialize(pack.as_mut_ptr(), 10);
            unsafe { pack.set_len(size); }
            
            state.ack = super::state::AckState::Awaiting;
            tokio::task::block_in_place(|| {
                let handle = tokio::runtime::Handle::current();
                handle.block_on(send_chan.send(pack)).unwrap();
            });
        }
        _ => { error!("Unexpected mes! Measurement is ignored!"); }
    }
}

pub fn fin_handle(state: CState, pack: FinPacket) {
    let mut state = state.lock().unwrap();
    if state.get_step_cnt() != pack.number_of_points { error!("Some mes points lost. Got {:?}, expected {:?}", state.get_step_cnt(), pack.number_of_points) }
    
    state.general = GeneralState::Idle;
}