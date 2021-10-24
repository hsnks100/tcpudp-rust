use async_std::io::WriteExt;
use async_std::net::{self, TcpListener, TcpStream};
use async_trait::async_trait;
use bytes::Bytes;
use futures::AsyncReadExt;
// use core::slice::SlicePattern;
// use core::slice::SlicePattern;
use std::net::SocketAddr;
use std::str;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::x086::{make_merge_packet, make_server_ack};

mod x086;
// extern crate x086;
// use x086;

// extern crate x086;
// use x086;
// mod x086;

fn main() -> anyhow::Result<()> {
    // let f = make_packet_header_body(3, 0x12, v);
    let v1 = x086::make_server_ack(5);
    let v2 = x086::make_server_ack(6);
    let mut v3 = Vec::new();
    v3.push(v1);
    v3.push(v2);
    let d = x086::make_merge_packet(v3);
    println!("packet: {:02X?}", d);
    async_std::task::block_on(entrypoint())?;
    Ok(())
}

// #[async_trait]
// trait Listener {
//     async fn listen(&mut self) -> anyhow::Result<()>;
// }
// enum StreamType {
//     Tcp(TcpStream),
//     Udp(Arc<Mutex<async_std::net::UdpSocket>>),
// }
enum StreamType<'a> {
    Tcp(&'a TcpStream),
    Udp(&'a async_std::net::UdpSocket),
}

async fn tcp_listen() -> anyhow::Result<()> {
    let tcp_listener = net::TcpListener::bind("0.0.0.0:4444").await?;
    while let Ok((tcp_stream, addr)) = tcp_listener.accept().await {
        println!("tcp connected {}", addr);
        let mut tcp_stream = tcp_stream;
        async_std::task::spawn(async move {
            let mut parser = Parser {
                session_seq: 1,
                id: "tcp processor".to_string(),
            };
            loop {
                let mut buf = [0; 10];
                // let tcp_stream = Arc::clone(&tcp_stream);
                // let tcp_stream = Arc::clone(&tcp_stream);
                // let mut tcp_stream = tcp_stream.lock().unwrap();
                // let parser = Arc::clone(&parser);
                match tcp_stream.read(&mut buf).await {
                    // socket closed
                    Ok(n) if n == 0 => {
                        println!("failed to read from socket;");
                        break;
                    }
                    Ok(n) => {
                        println!("recv tcp: {:?}", &buf[..n]);
                        println!("tcp write");
                        let bytes = Bytes::copy_from_slice(&buf[..n]);
                        // let parser = parser.lock().unwrap();
                        // parser.void().await;
                        parser
                            .processor(addr, bytes, &StreamType::Tcp(&tcp_stream))
                            .await;
                        // parser.lock().unwrap().processor(addr, bytes, StreamType::Tcp(tcp_stream.clone())).await;
                    }
                    Err(e) => {
                        println!("failed to read from socket; err = {:?}", e);
                        break;
                    }
                };
            }
        });
    }
    Ok(())
}
async fn udp_listen(ipport: String, init_seq: u16, parser_name: String) -> anyhow::Result<()> {
    let mut buffer = [0u8; 1024];
    let udp_listener = net::UdpSocket::bind(ipport).await?;

    // let u = Arc::clone(&udp_listener);
    // udp_listener.lock().unwrap().await?;
    let mut parser = Parser {
        session_seq: init_seq,
        id: parser_name.clone(),
    };
    while let Ok((size, addr)) = udp_listener.recv_from(&mut buffer).await {
        // println!("udp data {}, {:?}", addr, &buffer[..size]);
        let bytes = Bytes::copy_from_slice(&buffer[..size]);
        // let udp_listener2 = Arc::clone(&u);
        parser
            .processor(addr, bytes, &StreamType::Udp(&udp_listener))
            .await;
    }
    Ok(())
}
async fn send<'a>(
    stream_type: &StreamType<'a>,
    addr: SocketAddr,
    bytes: Bytes,
) -> anyhow::Result<()> {
    match stream_type {
        StreamType::Tcp(mut stream) => {
            stream.write_all(&bytes.slice(..)).await?;
        }
        StreamType::Udp(stream) => {
            // let stream = stream.lock().unwrap();
            // stream.send_to(b"hello", &addr).await;

            println!("->: {:X?}", &bytes.slice(..));
            stream.send_to(&bytes.slice(..), &addr).await?;
            // stream.lock().unwrap().send_to(&bytes.slice(..), &addr).await;
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

struct Parser {
    session_seq: u16,
    id: String,
}
impl Parser {
    async fn processor<'a>(
        &mut self,
        addr: SocketAddr,
        bytes: Bytes,
        stream_type: &StreamType<'a>,
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
                    let v1 = x086::make_server_ack(self.session_seq);
                    // println!("0x03...!!!!!!!!!!!!!!!!!!!!! {:02X?}", v1);
                    self.session_seq += 1;
                    let mut v3 = Vec::new();
                    v3.push(v1);
                    let sendData = make_merge_packet(v3);
                    // println!("0x03...!!!!222222222222 {:02X?}", sendData);
                    send(
                        stream_type,
                        addr,
                        Bytes::copy_from_slice(sendData.as_slice()),
                    )
                    .await;
                } else if msg_type == 0x04 {
                } else if msg_type == 0x06 {
                    let v1 = x086::make_server_ack(self.session_seq);
                    self.session_seq += 1;
                    let mut v3 = Vec::new();
                    v3.push(v1);
                    let sendData = make_merge_packet(v3);
                    send(
                        stream_type,
                        addr,
                        Bytes::copy_from_slice(sendData.as_slice()),
                    )
                    .await;
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

async fn entrypoint() -> anyhow::Result<()> {
    let tcp_fut = tcp_listen();
    let udp_fut = udp_listen("0.0.0.0:27888".to_string(),
     1818, "27888 parser".to_string());
    let udp_fut2 = udp_listen(
        "0.0.0.0:27999".to_string(),
        0x0,
        "27999 parser".to_string(),
    );
    futures::try_join!(tcp_fut, udp_fut, udp_fut2)?;
    Ok(())
}
