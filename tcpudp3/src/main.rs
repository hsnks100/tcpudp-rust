use async_std::channel::{Receiver, Sender, self};
use async_std::net::{self, TcpStream};
use async_std::task;
use async_std::io::WriteExt;
use futures::{AsyncRead, AsyncReadExt, Stream, StreamExt};
use std::sync::Arc;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use bytes::BytesMut;

enum Command {
    Send(SocketAddr),
    EXIT,
}

fn main() {
    async_std::task::block_on(start_service());
}

async fn start_service() -> anyhow::Result<()> {
    futures::join!(tcp_listener(), udp_listener());
    Ok(())
}

async fn tcp_listener() -> anyhow::Result<()> {
    let listener = net::TcpListener::bind("0.0.0.0:4444").await?;
    while let Ok((conn, addr)) = listener.accept().await {
        println!("tcp connected {}", addr);
        let (asm_sender, asm_recver) = channel::unbounded(); 
        async_std::task::spawn(tcp_send(conn.clone(), asm_recver));
        async_std::task::spawn(connection(addr, conn.clone(), asm_sender));
    }

    Ok(())
}

async fn processor( addr: SocketAddr, bytes: Bytes, sender: Sender<Command>) -> anyhow::Result<()> {
    println!("common processor: {}: {:?}", addr, bytes);
    // some logics...
    // 첫글자가 k 면 hello_world 보내주기.
    sender.send(Command::Send(addr)).await.unwrap();
    if bytes[0] == 'k' as u8 {
    }
    return Ok(())
}



async fn tcp_send(mut stream: TcpStream, recv: Receiver<Command>) {
    loop {
        match recv.recv().await.unwrap() {
            Command::Send(addr) => {
                stream.write_all(b"hello tcp").await;
            }, 
            Command::EXIT => {
                break;
            }, 
        }
    }
}

async fn udp_send(stream: Arc<net::UdpSocket>, r: Receiver<Command>) {
    loop {
        match r.recv().await.unwrap() {
            Command::Send(addr) => {
                stream.send_to(&Bytes::copy_from_slice(b"hello"), &addr).await; // .await 써야하는데...
            }, 
            Command::EXIT => {
                break;
            }, 
        }
        // println!("{}", r.recv().await.unwrap());
        // stream.send_to(
    }
}
async fn udp_listener() -> anyhow::Result<()> {
    let listener = Arc::new(net::UdpSocket::bind("0.0.0.0:4445").await?);
    let (asm_sender, asm_recver) = channel::unbounded(); 
    async_std::task::spawn(udp_send(Arc::clone(&listener), asm_recver));

    let mut buffer = [0u8; 10];
    while let Ok((size, addr)) = listener.recv_from(&mut buffer).await {
        println!("udp data {}, {:?}", addr, &buffer[..size]);
        processor(addr, Bytes::copy_from_slice(&buffer[..size]), asm_sender.clone()).await;
    }
    Ok(())
}
async fn connection(addr: SocketAddr, mut stream: TcpStream, sender: Sender<Command>) {
    let mut buf = [0; 10];
    loop {
        let n = match stream.read(&mut buf).await {
            // socket closed
            Ok(n) if n == 0 => {
                println!("failed to read from socket;");
                sender.send(Command::EXIT).await.unwrap();
                break;
            }
            Ok(n) => {
                println!("recv tcp: {:?}", &buf[..n]);
                // let (asm_sender, asm_recver) = channel::unbounded(); 
                processor(addr, Bytes::copy_from_slice(&buf[..n]), sender.clone()).await;
            }
            Err(e) => {
                println!("failed to read from socket; err = {:?}", e);
                break;
            }
        };
    }
}

