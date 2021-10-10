use async_std::channel::{unbounded, Receiver, Sender};
use async_std::net::{self, TcpStream};
use async_std::task;
use async_std::io::WriteExt;
use futures::{AsyncRead, AsyncReadExt, Stream, StreamExt};

use std::collections::HashMap;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use bytes::BytesMut;

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
        async_std::task::spawn(connection(addr, conn.clone()));
    }

    Ok(())
}

enum StreamType <'a>{
    Tcp(&'a TcpStream),
    Udp(&'a async_std::net::UdpSocket),
}

// async fn processor(addr: String, bytes: Bytes, mut stream: StreamType) -> anyhow::Result<()> {
// async fn processor<'a>(addr: SocketAddr, bytes: Bytes, stream: &'a StreamType) -> anyhow::Result<()> {

async fn send_bytes<'a> (addr: SocketAddr, stream: &StreamType<'a>, bytes: Bytes) -> anyhow::Result<()> {
    match stream {
        StreamType::Tcp(mut stream) => {
            println!("tcp write");
            stream.write_all(&bytes.slice(..)).await;
        },
        StreamType::Udp(stream) => {
            println!("udp write");
            stream.send_to(&bytes.slice(..), &addr).await;
        }
    }

    Ok(())
}
async fn processor<'a>( addr: SocketAddr, bytes: Bytes, stream: &StreamType<'a>) -> anyhow::Result<()> {
    println!("common processor: {}: {:?}", addr, bytes);
    // some logics...
    send_bytes(addr, stream, Bytes::copy_from_slice(b"hello_world")).await;
    return Ok(())
}

async fn udp_listener() -> anyhow::Result<()> {
    let listener = net::UdpSocket::bind("0.0.0.0:4445").await?;
    // let mut table = HashMap::<SocketAddr, Sender<Bytes>>::new();
    let mut buffer = [0u8; 10];
    while let Ok((size, addr)) = listener.recv_from(&mut buffer).await {
        println!("udp data {}, {:?}", addr, &buffer[..size]);
        processor(addr, Bytes::copy_from_slice(&buffer[..size]), &StreamType::Udp(&listener)).await;
    }
    Ok(())
}

async fn connection(addr: SocketAddr, mut stream: TcpStream) {
    let mut buf = [0; 10];
    loop {
        let n = match stream.read(&mut buf).await {
            // socket closed
            Ok(n) if n == 0 => {
                println!("failed to read from socket;");
                break;
            }
            Ok(n) => {
                println!("recv tcp: {:?}", &buf[..n]);
                processor(addr, Bytes::copy_from_slice(&buf[..n]), &StreamType::Tcp(&stream)).await;
            }
            Err(e) => {
                println!("failed to read from socket; err = {:?}", e);
                break;
            }
        };
    }
}
