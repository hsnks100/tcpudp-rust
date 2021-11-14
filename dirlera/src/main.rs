use async_std::net::{self, TcpListener, TcpStream};
use async_trait::async_trait;
use futures::AsyncReadExt;
// use core::slice::SlicePattern;
// use core::slice::SlicePattern;
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
// use async_lock::{Mutex};
// use std::sync::Mutex;
use crate::user_channel::*;

mod global;
mod network;
mod processor;
mod user_channel;
mod x086;

// use crate::lib;
// use crate::x086::{make_merge_packet, make_server_ack};

// use crate::*;
// use processor::*;
// extern crate x086;
// mod x086;
// use x086::*;

// extern crate x086;
// use x086;
// mod x086;

async fn entrypoint() -> anyhow::Result<()> {
    let tcp_fut = tcp_listen();
    // ;
    let udp_fut = udp_listen(
        "0.0.0.0:27888".to_string(),
        1818,
        "27888 parser".to_string(),
    );
    let udp_fut2 = udp_listen("0.0.0.0:27999".to_string(), 0x0, "27999 parser".to_string());
    futures::try_join!(tcp_fut, udp_fut, udp_fut2)?;
    Ok(())
}
fn main() -> anyhow::Result<()> {
    {
        // let mut uc = global::USERCHANNEL.lock().await;
        // uc.add_user(
        //     "1000".to_string(),
        //     Userstruct {
        //         id: "100".to_string(),
        //         name: "DIR-E".to_string(),
        //         ping: 3,
        //         connect_type: 3,
        //         player_status: 1,
        //     },
        // )?;
    }
    // let uc = USERCHANNEL.lock().unwrap();
    // let f = x086::make_packet_header_body(3, 0x12, v);
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

async fn tcp_listen() -> anyhow::Result<()> {
    let tcp_listener = net::TcpListener::bind("0.0.0.0:4444").await?;
    while let Ok((tcp_stream, addr)) = tcp_listener.accept().await {
        println!("tcp connected {}", addr);

        let mut tcp_stream = tcp_stream;
        async_std::task::spawn(async move {
            let mut parser = processor::Processor {
                session_seq: 1,
                id: "tcp processor".to_string(),
                x06_count: 0,
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
                            .process(addr, bytes, 
                                &network::StreamType::Tcp(&tcp_stream), 
                                Arc::clone(&global::USERCHANNEL))
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
    let mut parser = processor::Processor {
        session_seq: init_seq,
        id: parser_name.clone(),
        x06_count: 0,
    };
    while let Ok((size, addr)) = udp_listener.recv_from(&mut buffer).await {
        // println!("udp data {}, {:?}", addr, &buffer[..size]);
        let bytes = Bytes::copy_from_slice(&buffer[..size]);
        // let udp_listener2 = Arc::clone(&u);
        let vv = Arc::clone(&global::USERCHANNEL);
        parser
            .process(addr, bytes, 
                &network::StreamType::Udp(&udp_listener), 
                vv)
            .await;
    }
    Ok(())
}
