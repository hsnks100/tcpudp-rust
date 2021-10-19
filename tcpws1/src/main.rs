use std::{env, io::Error};

use async_std::net::{self, TcpListener, TcpStream};
use async_std::task;
use futures::{TryStreamExt, prelude::*};
use futures::join;
use futures_channel::mpsc::{unbounded, UnboundedSender, UnboundedReceiver};
use async_std::sync::{Arc, Mutex};

use async_tungstenite::tungstenite::protocol::Message;
use async_tungstenite::*;
use bytes::{Bytes, BytesMut, Buf, BufMut};
use std::net::SocketAddr;
use futures_util::stream::*;

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

async fn send_recver(recv:  Arc<Mutex<UnboundedReceiver<Message>>>, mut wss: SplitSink<WebSocketStream<TcpStream>, Message>) -> anyhow::Result<()> {
    loop {
        println!("loop start");    
        let mut data = recv.lock().await;
        let data = data.next().await.unwrap();
        if data.is_close() {
            break;
        }
        println!("Some!!: {:?}", data);
        let org = data.into_text().unwrap();
        
        let ksoo = Message::Text(org);
        wss.send(ksoo).await?;
    }
    println!("============ send_recver end =========");
    Ok(())
}

async fn ws_loop(addr: SocketAddr, mut wss_recv: SplitStream<WebSocketStream<TcpStream>>,  tx: UnboundedSender<Message>) -> anyhow::Result<()> {
    let parser = Parser{};
    loop {
        let resp = wss_recv.next().await;
        if resp.is_none() {
            return Ok(()) //should be error but anyway
        }
        
        let resp = resp.unwrap()?;
        let bytes = Bytes::copy_from_slice(resp.to_string().as_bytes());
        parser.processor(addr, bytes, StreamType::Ws(tx.clone())).await?;
        // tx.send(resp.clone()).await?;
        println!("{:?}", resp);
        if resp.is_close() {
            println!("1111112");
            break;
        }
    }
    println!("============ ws_loop end =========");
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
    let rx = Arc::new(Mutex::new(rx));
    // let i: i32 = read;
    let ws_handler = task::spawn(ws_loop(addr, read, tx.clone()));
    let ws_sender = task::spawn(send_recver(Arc::clone(&rx), write));
    join!(ws_handler, ws_sender);
    
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
                        parser.processor(addr, bytes, StreamType::Tcp(tcp_stream.clone())).await?;
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

enum StreamType {
    Tcp(TcpStream),
    Ws(UnboundedSender<Message>),
}

async fn send( stream_type: StreamType, addr: SocketAddr, bytes: Bytes) -> anyhow::Result<()> {
    match stream_type {
        StreamType::Tcp(mut stream) => {
            stream.write_all(&bytes.slice(..)).await?;
        },
        StreamType::Ws(mut stream) => {
            stream.send(Message::Binary(bytes.to_vec())).await?;
        },
    }
    Ok(())
    // Ok(())
}
struct Parser {}

impl Parser {
    
     async fn processor(&self, addr: SocketAddr, bytes: Bytes, stream_type: StreamType) -> anyhow::Result<()> {
        println!("parser processor {:?}, {:?}", addr, bytes);
        send(stream_type, addr, Bytes::copy_from_slice(b"i'm proc")).await?;
        // send(stream_type, addr, Bytes::copy_from_slice(b"i'm proc")).await;
        Ok(())
    }
}