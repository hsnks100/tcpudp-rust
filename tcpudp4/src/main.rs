use async_std::io::WriteExt;
use async_std::net::{self, TcpListener, TcpStream};
use futures::AsyncReadExt;
use async_std::task;

use std::sync::{Mutex, Arc};

use std::net::SocketAddr;

use async_trait::async_trait;
use bytes::Bytes;

fn main() {
    async_std::task::block_on(entrypoint());
}

async fn entrypoint() -> anyhow::Result<()> {
    let tcp_listener = net::TcpListener::bind("0.0.0.0:4444").await?;
    let udp_listener = net::UdpSocket::bind("0.0.0.0:4445").await?;

    let mut tcp = Tcp{tcp_listener: tcp_listener, tcp_stream: Option::None};
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

#[async_trait]
trait Listener {
    async fn listen(&mut self) -> anyhow::Result<()>;
    async fn send(&mut self, addr: SocketAddr, bytes: Bytes) ->anyhow::Result<()>;
    // tcp, udp 동시에 처리하는 프로세서.
    async fn processor(&mut self, addr: SocketAddr, bytes: Bytes) -> anyhow::Result<()> {
        if bytes[0] == 'k' as u8 {
            self.send(addr, Bytes::copy_from_slice(b"k-start")).await?;

        }
        self.send(addr, Bytes::copy_from_slice(b"i'am processor")).await?;
        Ok(())
    }
}

#[async_trait]
impl Listener for Tcp {
    async fn listen(&mut self) -> anyhow::Result<()> {
        while let Ok((tcp_stream, addr)) = self.tcp_listener.accept().await {
            println!("tcp connected {}", addr);
            // let self_copy = Arc::new(&self); //.clone();
            self.tcp_stream = Option::Some(tcp_stream.clone());
            let mut ee = Arc::new(Mutex::new(self));
            // let e3: i32 = ee;
            async_std::task::spawn(connection(addr, tcp_stream.clone(), Arc::clone(&ee))); // .await;
        }
        Ok(())
    }
    
    async fn send(&mut self, addr: SocketAddr, bytes: Bytes) ->anyhow::Result<()> {
        let ts = self.tcp_stream.clone();
        match ts {
            Option::Some(mut stream) => {
                println!("tcp send");
                stream.write_all(&bytes.slice(..)).await?
                // stream.write_all()
            },
            Option::None => {
                println!("tcp None");
            }
        }
        Ok(())
    }
}


#[async_trait]
impl Listener for Udp {
    async fn listen(&mut self) -> anyhow::Result<()> {
        let mut buffer = [0u8; 10];
        while let Ok((size, addr)) = self.0.recv_from(&mut buffer).await {
            println!("udp data {}, {:?}", addr, &buffer[..size]);
            let bytes = Bytes::copy_from_slice(&buffer[..size]);
            self.processor(addr, bytes).await?;
        }
        Ok(())
    }
    async fn send(&mut self, addr: SocketAddr, bytes: Bytes) ->anyhow::Result<()> {
        self.0.send_to(&bytes.slice(..), &addr).await?;
        Ok(())
    }
}

async fn connection(addr: SocketAddr, mut tcp_stream: TcpStream, tcp_socket: Arc<Mutex<&mut Tcp>>) -> anyhow::Result<()> {
    let mut buf = [0; 10];
    // self.test_int = 55;
    // self.tcp_stream = Some(tcp_stream.clone());
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
                let mut t = tcp_socket.lock().unwrap();
                t.send(addr, bytes).await?;
                // tcp_stream.write_all(&bytes.slice(..)).await?;
            }
            Err(e) => {
                println!("failed to read from socket; err = {:?}", e);
                break;
            }
        };
    }
    Ok(())
}