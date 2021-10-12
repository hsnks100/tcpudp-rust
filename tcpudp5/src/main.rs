use async_std::io::WriteExt;
use async_std::net::{self, TcpListener, TcpStream};
use futures::AsyncReadExt;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use bytes::Bytes;
use std::thread;

fn main() -> anyhow::Result<()> {
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

enum StreamType <'a>{
    Tcp(&'a TcpStream),
    Udp(&'a async_std::net::UdpSocket),
}


async fn tcp_listen() -> anyhow::Result<()> {
    let tcp_listener = net::TcpListener::bind("0.0.0.0:4444").await?;
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
                        parser.processor(addr, bytes, &StreamType::Tcp(&tcp_stream)).await;
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
async fn udp_listen() -> anyhow::Result<()> {
    let mut buffer = [0u8; 10];
    let udp_listener = net::UdpSocket::bind("0.0.0.0:4445").await?;

    // let u = Arc::clone(&udp_listener);
    // udp_listener.lock().unwrap().await?;
    while let Ok((size, addr)) = udp_listener.recv_from(&mut buffer).await {
        println!("udp data {}, {:?}", addr, &buffer[..size]);
        let bytes = Bytes::copy_from_slice(&buffer[..size]);
        let parser = Parser {};
        // let udp_listener2 = Arc::clone(&u);
        parser.processor(addr, bytes, &StreamType::Udp(&udp_listener)).await;
    }
    Ok(())
}
async fn send<'a>(stream_type: &StreamType <'a>, addr: SocketAddr, bytes: Bytes) {
    match stream_type {
        StreamType::Tcp(mut stream) => {
            stream.write_all(&bytes.slice(..)).await;
        },
        StreamType::Udp(stream) => {
            // let stream = stream.lock().unwrap();
            stream.send_to(b"hello", &addr).await;
            // stream.lock().unwrap().send_to(&bytes.slice(..), &addr).await;
        },
    }
    // Ok(())
}

struct Parser {}
impl Parser {
    
    async fn processor<'a>(&self, addr: SocketAddr, bytes: Bytes, stream_type: &StreamType<'a>) {
        println!("parser processor {:?}, {:?}", addr, bytes);
        send(stream_type, addr, Bytes::copy_from_slice(b"i'm proc")).await;
    }
}

async fn entrypoint() -> anyhow::Result<()> {
    let tcp_fut = tcp_listen();
    let udp_fut = udp_listen();
    futures::try_join!(tcp_fut, udp_fut)?;
    Ok(())
}
