use std::{io::{Read, Write}, ops::Deref};
use scanner_comms::packets::Packet;
use serialport::SerialPort;

mod coder;

enum State {
    Idle,
    Measure,
}

enum AckState {
    Send,
    Await,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    let mut state = State::Idle;
    let mut ack = AckState::Send;
    
    let mut mock_data = Vec::<u32>::new();
    let mut mock_iter = 0;
    
    let com_port = &args[1];
    let dur = args[2].parse::<u64>().unwrap();
    
    let mut port = serialport::new(com_port, 115_200).open_native().unwrap();
    
    port.set_timeout(std::time::Duration::from_secs(30)).unwrap();
    
    #[cfg(unix)]
    port.set_exclusive(false)
        .expect("Unable to set serial port exclusive to false");
        
    let mut buf = Vec::<u8>::new();
    loop {
        let mut tbuf = [0u8; 1];
        let _ = port.read(&mut tbuf).unwrap();
        buf.push(tbuf[0]);
        if tbuf[0] != 0x00 { continue; }
        
        match coder::decode_packet(&mut buf){
            Err(_) => {
                println!("Frame broken!");
                buf = Vec::<u8>::new();
                continue;
            },
            Ok(obj) => {
                match obj {
                    coder::PackType::Prog(pack) => {
                        match state {
                            State::Idle => {
                                state = State::Measure;
                                println!("Got scan request!");
                                mock_data = gen_data_points(pack.number_of_lines, pack.number_of_points);
                                let resp = scanner_comms::packets::packet_ok::OkPacket::new(123, 0x00, 0x00);
                                
                                let mut pack = Vec::with_capacity(10);
        
                                let size = resp.serialize(pack.as_mut_ptr(), 10);
                                unsafe { pack.set_len(size); }
                                port.write_all(&pack).unwrap();
                                
                                for element in mock_data {
                                    println!("Sending mock point");
                                    let mes = scanner_comms::packets::packet_mes::MesPacket::new(123, element);
                                    
                                    let mut pack = Vec::with_capacity(12);
        
                                    let size = mes.serialize(pack.as_mut_ptr(), 12);
                                    unsafe { pack.set_len(size); }
                                    port.write_all(&pack).unwrap();
                                    
                                    let mut tbuf = [0u8;10];
                                    port.read_exact(&mut tbuf).unwrap();
                                    let mut tbuf = Vec::<u8>::from(tbuf);
                                    
                                    let obj = coder::decode_packet(&mut tbuf).unwrap();
                                    match obj {
                                        coder::PackType::Ok(_) => println!("Got Ack!"),
                                        _ => panic!("Got something else than ok!"),
                                    }
                                    std::thread::sleep(std::time::Duration::from_millis(dur));
                                }
                            }
                            State::Measure => {
                                panic!("Critical error in comms, scan request sent while scanning!");
                            }
                        }
                    },  
                    _ => {println!("Unknown pack!");},
                }
            }
        }
        
    }
}


fn gen_data_points(lines: u8, points: u8) -> Vec<u32> {
    (1..=lines as u32 *points as u32 + 1).collect()
}