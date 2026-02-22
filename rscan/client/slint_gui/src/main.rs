// Copyright (C) 2024 pitau
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use std::{io::Write, sync::{Arc, Mutex}};

use log::{debug, error, info, warn};
use slint::{ComponentHandle, SharedString};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_serial::SerialPortBuilderExt;

use scanner_comms::{self, packets::{Packet, RotSide}};

slint::include_modules!();

mod state;
mod coder;
mod handlers;

const FRAME_END_TOKEN: u8 = 0x00;

type CState = Arc<Mutex<state::ClientState>>;



#[tokio::main]
async fn main() -> Result<(), slint::PlatformError> {
    
    env_logger::init();
    debug!("Main thread started");
    
    let args: Vec<String> = std::env::args().collect();
    
    //let com_port = "/dev/pts/7";
    let com_port = &args[1];
    let target_file = &args[2];
    let baud_rate = args[3].parse::<u32>().unwrap();
    
    //let port = serial2_tokio::SerialPort::open(com_port, 115_200).unwrap();
    let target_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(target_file)
        .unwrap();
    
    let mut port = tokio_serial::new(com_port, baud_rate).open_native_async().unwrap();
    
    #[cfg(unix)]
    port.set_exclusive(false)
        .expect("Unable to set serial port exclusive to false");
        
    info!("Opened port: {:?}", com_port);
    
    let client_state: CState = Arc::new(Mutex::new(state::ClientState::new(target_file)));

    let (mut port_rx, mut port_tx) = tokio::io::split(port);
    
    let (send_chan, mut client_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(1);
    //let (device_tx, recv_chan) = tokio::sync::mpsc::channel::<Vec<u8>>(1);
    
    let (progress_tx, mut progress_rx) = tokio::sync::mpsc::channel::<u16>(32);
    
    debug!("Channels initialized!");
    //let l_port = port.clone();
    let state_clone = client_state.clone();
    tokio::spawn(async move {
        debug!("Spawned send thread");
        loop {
            let pack = client_rx.recv().await.unwrap();
            
            debug!("Got pack: {:?}", pack);
            
            //Skip saving if error:
            if pack[4] != 2 {
                let mut state = state_clone.lock().unwrap();
                state.last_pack = pack.clone();
            }
            
            debug!("Sent packet: {:?}", pack);
            
            port_tx.write(& pack).await.unwrap();
        }
    });

    let send_chan_clone = send_chan.clone();
    let state_clone = client_state.clone();
    tokio::spawn(async move {
        debug!("Spawned listener thread");
        let mut buf: Vec<u8> = Vec::<u8>::new();
        loop {
            let val = port_rx.read_u8().await;
            
            match val {
                Ok(val) => {
                    buf.push(val);
                    if val != FRAME_END_TOKEN { continue; }
                    else {
                        debug!("Got frame: {:?}", buf);
                        let ret = coder::decode_packet(&mut buf);
                        match ret {
                            Err(_) => {
                                warn!("Frame borked!");
                                buf = Vec::<u8>::new();
                                continue;
                            }
                            Ok(obj) => {
                                match obj {
                                    coder::PackType::Ok(pack) => handlers::ok_pack(state_clone.clone(), pack),
                                    coder::PackType::Err(pack) => handlers::err_handle(state_clone.clone(), pack, send_chan_clone.clone()),
                                    coder::PackType::Mes(pack) => handlers::mes_handle(state_clone.clone(), pack, send_chan_clone.clone(), progress_tx.clone()),
                                    coder::PackType::Fin(pack) => handlers::fin_handle(state_clone.clone(), pack),
                                    _ => println!("Other pack!"),
                                };
                            }
                        }
                        println!("Decoded {:?}", buf);
                        buf = Vec::<u8>::new();
                    }
                }
                Err(val) => {
                    error!("We got error in send channel: {:?}", val);
                }
            }
        }
    });

    let ui = MainAppWindow::new()?;
    
    let tx_clone = send_chan.clone();
    
    ui.on_send_abort_pack(move || {
        let abort = scanner_comms::packets::packet_abort::AbortPacket::new(123);
        let mut pack = Vec::with_capacity(7);
        
        let size = abort.serialize(pack.as_mut_ptr(), 11);
        unsafe { pack.set_len(size); }
        let tx_clone = tx_clone.clone();
        let _ = slint::spawn_local(async move { tx_clone.send(pack).await.unwrap(); });
    });
    
    let tx_clone = send_chan.clone();
    ui.on_pass_z_rot(move |number: SharedString| {
        match number.parse::<i16>() {
            Err(e) => warn!("Casting step value ended with error: {:?}", e)
,           Ok(steps) => {
                let mut mov = scanner_comms::packets::packet_mov::MovPacket::new(123, scanner_comms::packets::Axis::Horizon, RotSide::Clockwise, 0);
                
                let mut pack: Vec<u8> = Vec::with_capacity(11);
                
                if (0..=200).contains(&steps) {
                    info!("Got {:?} steps Clockwise", steps);
                    mov.steps = steps.try_into().expect("Something went very wrong!");
                    mov.side = RotSide::Clockwise;
                }
                else if (-200..0).contains(&steps) {
                    info!("Got {:?} steps Counter-clockwise", steps);
                    mov.steps = steps.abs().try_into().expect("Something went very wrong!");
                    mov.side = RotSide::CounterClockwise;
                }
                else { warn!("Value out of range for the device!"); return; };
                
                let size = mov.serialize(pack.as_mut_ptr(), 11);
                unsafe { pack.set_len(size); }
                let tx_clone = tx_clone.clone();
                let _ = slint::spawn_local(async move { tx_clone.send(pack).await.unwrap(); });
            }
        }
    });
    
    let tx_clone = send_chan.clone();
    ui.on_pass_x_rot(move |number: SharedString| {
        match number.parse::<i16>() {
            Err(e) => warn!("Casting step value ended with error: {:?}", e)
,           Ok(steps) => {
                let mut mov = scanner_comms::packets::packet_mov::MovPacket::new(123, scanner_comms::packets::Axis::Azimuth, RotSide::Clockwise, 0);
                
                let mut pack: Vec<u8> = Vec::with_capacity(11);
                
                if (0..=200).contains(&steps) {
                    info!("Got {:?} steps Clockwise", steps);
                    mov.steps = steps.try_into().expect("Something went very wrong!");
                    mov.side = RotSide::Clockwise;
                }
                else if (-200..0).contains(&steps) {
                    info!("Got {:?} steps Counter-clockwise", steps);
                    mov.steps = steps.abs().try_into().expect("Something went very wrong!");
                    mov.side = RotSide::CounterClockwise;
                }
                else { warn!("Value out of range for the device!"); return; };
                
                let size = mov.serialize(pack.as_mut_ptr(), 11);
                unsafe { pack.set_len(size); }
                let tx_clone = tx_clone.clone();
                let _ = slint::spawn_local(async move { tx_clone.send(pack).await.unwrap(); });
            }
        }
    });
    
    let state_clone = client_state.clone();
    ui.on_read_steps_update(move |number: SharedString|{
        debug!("Updated string to: {:?}", number);
        match number.parse::<u8>() {
            Err(e) => { warn!("Value cannto be cast due to: {:?}", e); return; },
            Ok(steps) => {
                let mut state = state_clone.lock().unwrap();
                state.set_steps(steps);
            }
        }
    });
    
    let state_clone = client_state.clone();
    ui.on_read_lines_update(move |number: SharedString|{
        debug!("Updated string to: {:?}", number);
        match number.parse::<u8>() {
            Err(e) => { warn!("Value cannto be cast due to: {:?}", e); return; },
            Ok(steps) => {
                let mut state = state_clone.lock().unwrap();
                state.set_lines(steps);
            }
        }
    });
    
    let state_clone = client_state.clone();
    let tx_clone = send_chan.clone();
    ui.on_send_prog_pack(move || {
        let mut state = state_clone.lock().unwrap();
        let abort = scanner_comms::packets::packet_prog::ProgPacket::new(123, state.get_steps(), state.get_lines());
        let mut pack = Vec::with_capacity(11);
        
        let size = abort.serialize(pack.as_mut_ptr(), 11);
        unsafe { pack.set_len(size); }
        let tx_clone = tx_clone.clone();
        let _ = slint::spawn_local(async move { tx_clone.send(pack).await.unwrap(); });
        
        let lines = state.get_lines();
        let steps = state.get_steps();
        state.ack = state::AckState::Awaiting;
        state.general = state::GeneralState::Programming;
        state.out_file.write_all(&[0x01, 0x01, lines, steps]).unwrap();
    });
    
    let ui_handle = ui.as_weak();
    let state_clone = client_state.clone();
    
    tokio::spawn(async move {
        loop {
            let raw_progress = progress_rx.recv().await.unwrap();
            let state = state_clone.lock().unwrap();
            let total = state.get_total_steps();
            let progress = raw_progress as f32 / state.get_total_steps() as f32;
            ui_handle.upgrade_in_event_loop(move |handle| {
                handle.set_progress(progress);
                handle.set_raw_progress(SharedString::from(format!("{:?}/{:?}", raw_progress, total)));
            }).unwrap();
        }
    });
    
    ui.run()?;
    return Ok(());
    
    /*
    loop {
        let testpack = scanner_comms::packets::packet_ok::OkPacket::new(123);
        
        let mut pack = Vec::with_capacity(9);
        
        let size = testpack.serialize(pack.as_mut_ptr(), 9);
        std::thread::sleep(std::time::Duration::from_millis(2000));
        println!("Call tx!");
        
        println!("Size: {:?}", size);
        println!("Da packet: {:?}", pack);
        
        unsafe { pack.set_len(size); }
        
        println!("Da packet: {:?}", pack);
        
        send_chan.send(pack).await.unwrap();
    }
    */
}
