

use std::sync::{Arc, Mutex};

use scanner_comms::{self, packets::Packet};

use tokio::{io::{AsyncReadExt, AsyncWriteExt, WriteHalf}, sync::mpsc::Sender};
use tokio_serial::{SerialPortBuilderExt, SerialStream};

mod coder;

enum State {
        Idle,
        Mes,
    }

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    //let com_port = "/dev/pts/7";
    let com_port = &args[1];
    
    //let port = serial2_tokio::SerialPort::open(com_port, 115_200).unwrap();
    
    let mut port = tokio_serial::new(com_port, 115_200).open_native_async().unwrap();
    
    let (mut port_rx, mut port_tx) = tokio::io::split(port);
    
    let (send_chan, mut client_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(1);
    let (device_tx, recv_chan) = tokio::sync::mpsc::channel::<Vec<u8>>(1);
    
    #[cfg(unix)]
    port.set_exclusive(false)
        .expect("Unable to set serial port exclusive to false");
        
    tokio::spawn(async move {
        
        loop {
            let pack = client_rx.recv().await.unwrap();
            
            println!("sending pack!");
            
            //let mut port = port.lock().unwrap();
            
            port_tx.write(& pack).await.unwrap();
        }
    });
        
    tokio::spawn(async move {
        let mut buf: Vec<u8> = Vec::<u8>::new();
        loop {
            let val = port_rx.read_u8().await;
                
            match val {
                Ok(val) => {
                    buf.push(val);
                    if val == 0 {
                        // Place decode code here!
                        let ret = coder::decode_packet(&mut buf);
                        match ret {
                            Err(_) => {
                                buf = Vec::<u8>::new();
                                continue;
                            }
                            Ok(obj) => {
                                match obj {
                                    coder::PackType::Ok(pack) => println!("Decoded {:?}", pack.header.crc),
                                    coder::PackType::Err(pack) => println!("Decoded error {:?} in packet {:?}", pack.error as u8, pack.packet_id),
                                    coder::PackType::Prog(pack) => { tokio::spawn(handle_prog(send_chan.clone(), pack)); },
                                    coder::PackType::Fin(pack) => println!("Decoded fin"),
                                    _ => println!("Other pack!"),
                                };
                            }
                        }
                        println!("Decoded {:?}", buf);
                        buf = Vec::<u8>::new();
                    }
                }
                Err(val) => {
                    println!("We got error {:?}", val);
                }
            }
            
            //device_tx.send(Vec::<u8>::from(buf)).await.unwrap();
            
            //Clear recv
            //let _ = recv_chan.recv().await.unwrap();
        }
    });
}

async fn handle_prog(mut tx_chan: Sender<Vec<u8>>, pack: scanner_comms::packets::packet_prog::ProgPacket) {
    println!("Got Prog!");
    let ok_ret = scanner_comms::packets::packet_ok::OkPacket::new(pack.header.packet_id);
    let mut buf: Vec<u8> = Vec::with_capacity(scanner_comms::packets::packet_ok::OkPacket::size_of() + 2);
    
    let size = ok_ret.serialize(buf.as_mut_ptr(), 9);
    unsafe { buf.set_len(size); }
    tx_chan.send(buf).await.unwrap();
    
}
