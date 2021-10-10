use async_std::io::WriteExt;
use async_std::net::{self, TcpListener, TcpStream};
use futures::AsyncReadExt;
use async_std::task;

use std::sync::Arc;

use std::net::SocketAddr;

use async_trait::async_trait;
use bytes::Bytes;

fn main() {
    async_std::task::block_on(main2());
}

async fn main2() -> anyhow::Result<()> {
    let tcp_listener = net::TcpListener::bind("0.0.0.0:4444").await?;
    let udp_listener = net::UdpSocket::bind("0.0.0.0:4445").await?;

    let tcp = Tcp{tcp_listener: tcp_listener, tcp_stream: Option::None, test_int: 33};
    let udp = Udp(udp_listener);

    let tcp_fut = Arc::new(tcp).listen();
    let udp_fut = Arc::new(udp).listen();
    futures::try_join!(tcp_fut, udp_fut)?;
    Ok(())
}
#[async_trait]
trait Listener {
    async fn listen(self: Arc<Self>) -> anyhow::Result<()>;
    async fn send(&self, addr: SocketAddr, bytes: Bytes);
    async fn processor(&self, addr: SocketAddr, bytes: Bytes) {
        println!("default imp");
    }
    async fn connection(&mut self, addr: SocketAddr, mut tcp_stream: TcpStream) -> anyhow::Result<()> {
        Ok(())
    }
}

struct Tcp {
    tcp_listener: TcpListener,
    tcp_stream: Option<TcpStream>,
    test_int: i32
}
struct Udp(async_std::net::UdpSocket);

async fn tt(t: Arc<Tcp>, addr: SocketAddr, tcp_stream: TcpStream) {
    // 이거 컴파일 해결 방법좀... ㅠ.ㅠ
    t.connection(addr, tcp_stream.clone()).await;
}

#[async_trait]
impl Listener for Tcp {
    async fn listen(self: Arc<Self>) -> anyhow::Result<()> {
        while let Ok((tcp_stream, addr)) = Arc::clone(&self).tcp_listener.accept().await {
            println!("tcp connected {}", addr);
            let self_copy = Arc::new(&self); //.clone();
            async_std::task::spawn(tt(Arc::clone(&self_copy), addr, tcp_stream.clone())); // .await;
        }
        Ok(())
    }
    async fn connection(&mut self, addr: SocketAddr, mut tcp_stream: TcpStream) -> anyhow::Result<()> {
        let mut buf = [0; 10];
        self.test_int = 55;
        self.tcp_stream = Some(tcp_stream.clone());
        loop {
            let n = match tcp_stream.read(&mut buf).await {
                // socket closed
                Ok(n) if n == 0 => {
                    println!("failed to read from socket;");
                    break;
                }
                Ok(n) => {
                    println!("recv tcp: {:?}", &buf[..n]);
                    println!("tcp write");
                    let bytes = Bytes::copy_from_slice(&buf[..n]);
                    tcp_stream.write_all(&bytes.slice(..)).await?;
                }
                Err(e) => {
                    println!("failed to read from socket; err = {:?}", e);
                    break;
                }
            };
        }
        Ok(())
    }
    async fn send(&self, addr: SocketAddr, bytes: Bytes) {
    }
}


#[async_trait]
impl Listener for Udp {
    async fn listen(self: Arc<Self>) -> anyhow::Result<()> {
        let mut buffer = [0u8; 10];
        while let Ok((size, addr)) = self.0.recv_from(&mut buffer).await {
            println!("udp data {}, {:?}", addr, &buffer[..size]);
            let bytes = Bytes::copy_from_slice(&buffer[..size]);
            self.processor(addr, bytes).await;
        }
        Ok(())
    }
    async fn send(&self, addr: SocketAddr, bytes: Bytes) {
        self.0.send_to(&bytes.slice(..), &addr).await;
    }
}

