use std::{env, io::Error};

use async_std::net::{self, TcpListener, TcpStream};
use async_std::task;
use futures::{TryStreamExt, prelude::*};
use log::info;
// use async_std::io::WriteExt;
use futures::join;
use futures::prelude::*;
// use futures::{
//     channel::mpsc::{unbounded, UnboundedSender},
//     future, pin_mut,
// };
use futures_channel::mpsc::{unbounded, UnboundedSender, UnboundedReceiver};

// use async_std::net::{TcpListener, TcpStream};
// use async_std::task;
use async_std::sync::{Arc, Mutex};

use async_tungstenite::tungstenite::protocol::Message;

use bytes::{Bytes, BytesMut, Buf, BufMut};
use std::net::SocketAddr;

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

async fn send_recver(mut recv:  UnboundedReceiver<Message>) -> anyhow::Result<()> {
    loop {
        
        let data = recv.try_next()?;
        match data {
            Option::Some(d) => {
                println!("recv!!: {}", d);
                break;
            },
            Option::None => {
                println!("none!!");
                break;
            }
        };
        
    }
    println!("send_recver exit");
    // recv.for_each(|result| {
    //     println!("Got: {}", result);
    //     Ok(())
    // });
    // loop {
    //     // recv
    //     // recv.recv().await.unwrap();
    //     println!("recv!!");
    // }
    Ok(())
}
async fn accept_connection(stream: TcpStream) -> anyhow::Result<()> {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    println!("Peer address: {}", addr);

    let ws_stream = async_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    println!("New WebSocket connection: {}", addr);

    let (write, read) = ws_stream.split();
    let (tx, rx) = unbounded::<Message>(); 
    
    let sender = send_recver(rx);
    let fut = read.try_for_each(move |msg| {
        
        println!("dddd {:?}", msg);
        if msg.is_close() {
            println!("socket is closed");
        } else if msg.is_text() {
            let b = tx.is_closed();
            println!("is_close2? : {}", b);
            tx.unbounded_send(Message::Text("power".to_string())).unwrap(); // 이거 패닉나는데... 
            println!("Received a message from {}: {}", "hi", msg.to_text().unwrap());
        }
        future::ok(())
    });
    join!(fut, sender);
    
    println!("disconnect??");
    Ok(())
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

    task::block_on(run())?;
    println!("exit");
    Ok(())
}

struct Parser {}
impl Parser {
    
    async fn processor(&self, addr: SocketAddr, bytes: Bytes) {
        println!("parser processor {:?}, {:?}", addr, bytes);
        // send(stream_type, addr, Bytes::copy_from_slice(b"i'm proc")).await;
    }
}
