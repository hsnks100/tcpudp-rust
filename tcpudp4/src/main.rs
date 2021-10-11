use async_std::io::WriteExt;
use async_std::net::{self, TcpListener, TcpStream};
use futures::AsyncReadExt;
use std::net::SocketAddr;

use async_trait::async_trait;
use bytes::Bytes;

fn main() -> anyhow::Result<()> {
    async_std::task::block_on(entrypoint())?;
    Ok(())
}

async fn entrypoint() -> anyhow::Result<()> {
    let tcp_listener = net::TcpListener::bind("0.0.0.0:4444").await?;
    let udp_listener = net::UdpSocket::bind("0.0.0.0:4445").await?;

    let mut tcp = Tcp {
        tcp_listener: tcp_listener,
        tcp_stream: Option::None,
    };
    let mut udp = Udp(udp_listener);

    let tcp_fut = tcp.listen();
    let udp_fut = udp.listen();
    futures::try_join!(tcp_fut, udp_fut)?;
    Ok(())
}

struct Tcp {
    tcp_listener: TcpListener,
    tcp_stream: Option<TcpStream>,
}
struct Udp(async_std::net::UdpSocket);

// enum SocketType {
//     async_std::net::UdpSocket(ss),
    
// }


#[async_trait]
trait Listener {
    async fn listen(&mut self) -> anyhow::Result<()>;   
}

#[async_trait]
impl Listener for Tcp {
    async fn listen(&mut self) -> anyhow::Result<()> {
        while let Ok((tcp_stream, addr)) = self.tcp_listener.accept().await {
            println!("tcp connected {}", addr);
            // let self_copy = Arc::new(&self); //.clone();
            self.tcp_stream = Option::Some(tcp_stream.clone());
            // let ee = Arc::new(Mutex::new(self));
            // let e3: i32 = ee;
            let parser = Parser{};
            async_std::task::spawn(connection(addr, tcp_stream.clone(), parser)); // .await;
        }
        Ok(())
    }
}

enum StreamType <'a>{
    Tcp(&'a TcpStream),
    Udp(&'a async_std::net::UdpSocket),
}

async fn sender<'a>(st: &StreamType<'a>, addr: SocketAddr, bytes: Bytes) ->anyhow::Result<()> {
    match st {
        StreamType::Tcp(mut stream) => {
            stream.write_all(&bytes.slice(..)).await?;
        },
        StreamType::Udp(stream) => {
            stream.send_to(&bytes.slice(..), &addr).await?;
        },
    }
    Ok(())
}
#[async_trait]
impl Listener for Udp {
    async fn listen(&mut self) -> anyhow::Result<()> {
        let mut buffer = [0u8; 10];
        while let Ok((size, addr)) = self.0.recv_from(&mut buffer).await {
            println!("udp data {}, {:?}", addr, &buffer[..size]);
            let bytes = Bytes::copy_from_slice(&buffer[..size]);
            let parser = Parser{};
            
            parser.processor(addr, bytes, &StreamType::Udp(&self.0)).await?;            
        }
        Ok(())
    }
}

async fn connection(
    addr: SocketAddr,
    mut tcp_stream: TcpStream,
    parser: Parser,
) -> anyhow::Result<()> {
    let mut buf = [0; 10];
    // self.test_int = 55;
    // self.tcp_stream = Some(tcp_stream.clone());
    loop {
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
                parser.processor(addr, bytes, &StreamType::Tcp(&tcp_stream)).await?;
            }
            Err(e) => {
                println!("failed to read from socket; err = {:?}", e);
                break;
            }
        };
    }
    Ok(())
}

struct Parser {

}
impl Parser {
    async fn processor<'a>(&self, addr: SocketAddr, bytes: Bytes, stream_type: &StreamType<'a>) -> anyhow::Result<()> 
    {
        println!("parser processor {:?}, {:?}", addr, bytes);
        sender(stream_type, addr, Bytes::copy_from_slice(b"i'm proc")).await?;
        Ok(())        
    }
}