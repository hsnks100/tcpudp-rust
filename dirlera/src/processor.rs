use async_std::io::WriteExt;
use bytes::Bytes;
use std::net::SocketAddr;
use std::str;

use async_std::net::{self, TcpListener, TcpStream};
use async_trait::async_trait;
use futures::AsyncReadExt;
// use core::slice::SlicePattern;
// use core::slice::SlicePattern;
use crate::*;
// use async_lock::Mutex;
use std::sync::{Arc, Mutex};

use std::thread;
// use network;
// use crate::network;
// use crate::x086::{make_merge_packet, make_server_ack};
// extern crate x086;

// pub struct ksoostruct2 {}

// impl ksoostruct2 {
//     pub fn test() {
//         x086::make_server_ack(5);
//     }
// }

// use crate::x086::*;
pub async fn send<'a>(
    stream_type: &network::StreamType<'a>,
    addr: SocketAddr,
    bytes: Bytes,
) -> anyhow::Result<()> {
    match stream_type {
        network::StreamType::Tcp(mut stream) => {
            stream.write_all(&bytes.slice(..)).await?;
        }
        network::StreamType::Udp(stream) => {
            let v = bytes.slice(..);
            let v = v.to_vec();
            println!("->: {:02X?}", v);
            let s = match str::from_utf8(v.as_slice()) {
                Ok(v) => v,
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
            println!("->(ascii): {}", s);
            stream.send_to(&bytes.slice(..), &addr).await?;
        }
    }
    Ok(())
}

#[repr(C, packed)]
#[derive(Debug)]
struct ProtocolHeader {
    n: u8,
    seq: u16,
    length: u16,
}

pub struct Processor {
    pub session_seq: u16,
    pub id: String,
    pub x06_count: u32,
}
impl Processor {
    pub async fn process<'a>(
        &mut self,
        addr: SocketAddr,
        bytes: Bytes,
        stream_type: &crate::network::StreamType<'a>,
        uc: Arc<Mutex<Userchannel>>,
    ) {
        println!("[{}] parser processor {:?}, {:X?}", self.id, addr, bytes);
        let v = bytes.slice(..);
        // v = 3;
        let utf8 = String::from_utf8(v.to_vec());
        if let Ok(t) = utf8 {
            if t.len() >= 4 && t[..4] == "PING".to_string() {
                println!("PING");
                send(
                    stream_type,
                    addr,
                    Bytes::copy_from_slice("PONG\u{0}".as_bytes()),
                )
                .await;
            } else if t.len() >= 5 && t[..5] == "HELLO".to_string() {
                println!("HELLOD00D");
                send(
                    stream_type,
                    addr,
                    Bytes::copy_from_slice("HELLOD00D27999\u{0}".as_bytes()),
                )
                .await;
            } else {
                if v.len() < 5 {
                    return;
                }
                let s: ProtocolHeader = unsafe { std::ptr::read(v.as_ptr() as *const _) };
                let s2 = bytes.slice(5..5 + s.length as usize);
                // println!("struct: {:?}", s);
                // println!("body: {:?}", s2);
                let msg_type = s2.slice(..1).to_vec()[0];
                let msg_body = s2.slice(1..).to_vec();
                println!(
                    "====================== type: {:02X} =============",
                    msg_type
                );
                if msg_type == 0x03 {
                    let mut sort: Vec<_> = msg_body.split(|i| *i == 0).collect();
                    // println!("split: {:?}", sort);
                    let sort0 = sort[0];
                    // let ii =
                    let nick = str::from_utf8(sort[0]).unwrap();
                    let emul = str::from_utf8(sort[1]).unwrap();
                    let conn_type = sort[2][0];
                    println!("nick: {}, emul: {}, conn_type: {}", nick, emul, conn_type);
                    // let mut uc = global::USERCHANNEL.lock().unwrap();
                    {
                        let mut uc = uc.lock().unwrap(); // .unwarp(); // await;
                        let next_sess = uc.get_next_sess_key();
                        uc.add_user(
                            addr.to_string(),
                            Userstruct {
                                id: next_sess,
                                name: nick.to_string(),
                                ping: 3,
                                connect_type: conn_type as u32,
                                player_status: 1,
                            },
                        );
                    }
                    let v1 = x086::make_server_ack(self.session_seq);
                    // println!("0x03...!!!!!!!!!!!!!!!!!!!!! {:02X?}", v1);
                    self.session_seq += 1;
                    let mut v3 = Vec::new();
                    v3.push(v1);
                    let sendData = x086::make_merge_packet(v3);
                    println!("=>ACK");
                    send(
                        stream_type,
                        addr,
                        Bytes::copy_from_slice(sendData.as_slice()),
                    )
                    .await;
                } else if msg_type == 0x04 {
                } else if msg_type == 0x06 {
                    self.x06_count += 1;
                    println!("x06_count: {}", self.x06_count);
                    if self.x06_count <= 6 {
                        let v1 = x086::make_server_ack(self.session_seq);
                        self.session_seq += 1;
                        let mut v3 = Vec::new();
                        v3.push(v1);
                        let sendData = x086::make_merge_packet(v3);
                        println!("=>ACK");
                        send(
                            stream_type,
                            addr,
                            Bytes::copy_from_slice(sendData.as_slice()),
                        )
                        .await;
                    } else {
                        self.session_seq += 1;
                        let v1 = x086::make_status_packet(self.session_seq);
                        let mut v3 = Vec::new();
                        v3.push(v1);
                        let sendData = x086::make_merge_packet(v3);
                        println!("=>STATUS");
                        send(
                            stream_type,
                            addr,
                            Bytes::copy_from_slice(sendData.as_slice()),
                        )
                        .await;

                        // let mut uc = global::USERCHANNEL.lock().unwrap();
                        // let u = uc.get_user(addr.to_string());
                        // if u.is_some() {
                        //     let u = Some(u);
                        //     self.session_seq += 1;
                        //     let v1 = x086::make_user_join_packet(self.session_seq, addr.to_string());
                        //     let mut v3 = Vec::new();
                        //     v3.push(v1);
                        //     let sendData = x086::make_merge_packet(v3);
                        //     println!("=>JOINED PACKET");
                        //     send(
                        //         stream_type,
                        //         addr,
                        //         Bytes::copy_from_slice(sendData.as_slice()),
                        //     )
                        //     .await;
                        // }
                    }
                } else {
                    println!("this packet is need to process: {:?}", s2.to_vec());
                }
            }
        }
        println!("=========================================");
        // let v = String::from_utf8(v.
        // send(stream_type, addr, Bytes::copy_from_slice(b"i'm proc")).await;
    }
}
