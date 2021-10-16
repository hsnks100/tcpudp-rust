use std::{env, io::Error};

use async_std::task;
use futures::prelude::*;
use log::info;
use async_std::io::WriteExt;
use async_std::net::{self, TcpListener, TcpStream};
use futures::AsyncReadExt;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use bytes::Bytes;
use std::thread;
use async_tungstenite::tungstenite::protocol::Message;
use async_std::channel::{unbounded, Receiver, Sender};

use futures::{future, pin_mut, StreamExt};
use futures::channel::mpsc;

async fn run() -> anyhow::Result<()> {
    let tcp_fut = tcp_listen();
    let ws_fut = ws_listen();
    futures::try_join!(tcp_fut, ws_fut)?;

    Ok(())
    
}

async fn ws_listen() -> anyhow::Result<()> {
    let _ = env_logger::try_init();
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    // info!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        task::spawn(accept_connection(stream));
    }

    Ok(())
}

async fn accept_connection(stream: TcpStream) {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    println!("Peer address: {}", addr);

    let ws_stream = async_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    println!("New WebSocket connection: {}", addr);

    let (write, read) = ws_stream.split();
    
    // read.forward(write).await;
    let fut = read.try_for_each(|msg| {
        println!("Received a message from {}: {}", "hi", msg.to_text().unwrap());
        // let mut m = &mut Message::Text("bbong".to_string());
        // read.forward(m);
        let (stdin_tx, stdin_rx) = mpsc::unbounded();
        stdin_tx.unbounded_send(Message::Text("power".to_string())).unwrap();
        future::ok(())
    }).await;
    
    // future::select(fut).await;

    // println!("recv: {}", read.to_text().unwrap());
    // read.forward(write)
    //     .await
    //     .expect("Failed to forward message")
}

async fn tcp_listen() -> anyhow::Result<()> {
    let tcp_listener = net::TcpListener::bind("0.0.0.0:8079").await?;
    while let Ok((tcp_stream, addr)) = tcp_listener.accept().await {
        println!("tcp connected {}", addr);
        
        let mut tcp_stream = tcp_stream;
        
        async_std::task::spawn(async move {
            let parser = Parser {};
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
                        // parser.processor(addr, bytes, &StreamType::Tcp(&tcp_stream)).await;
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

fn main() -> anyhow::Result<()>{

    task::block_on(run())
    // Ok(())
}

struct Parser {}
impl Parser {
    
    async fn processor(&self, addr: SocketAddr, bytes: Bytes) {
        println!("parser processor {:?}, {:?}", addr, bytes);
        // send(stream_type, addr, Bytes::copy_from_slice(b"i'm proc")).await;
    }
}
