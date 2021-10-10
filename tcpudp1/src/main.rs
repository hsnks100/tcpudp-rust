use async_std::channel::{unbounded, Receiver, Sender};
use async_std::net::{self, TcpStream};
use async_std::task;

use futures::{AsyncRead, AsyncReadExt, Stream, StreamExt};

use std::collections::HashMap;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use bytes::BytesMut;

fn main() {
    task::block_on(start_service());
}

async fn start_service() -> anyhow::Result<()> {
    futures::join!(tcp_listener(), udp_listener());

    Ok(())
}

async fn tcp_listener() -> anyhow::Result<()> {
    let listener = net::TcpListener::bind("0.0.0.0:4444").await?;
    while let Ok((conn, addr)) = listener.accept().await {
        println!("tcp connected {}", addr);
        task::spawn(handle_generic_connection(SocketStream::from_tcp_stream(
            conn,
        )));
    }

    Ok(())
}

async fn udp_listener() -> anyhow::Result<()> {
    let listener = net::UdpSocket::bind("0.0.0.0:4445").await?;

    let mut table = HashMap::<SocketAddr, Sender<Bytes>>::new();
    let mut buffer = [0u8; 4096];
    while let Ok((size, addr)) = listener.recv_from(&mut buffer).await {
        println!("udp data {}", addr);

        let send = if let Some(send) = table.get(&addr) {
            send.clone()
        } else {
            let (send, recv) = unbounded();
            table.insert(addr, send.clone());

            task::spawn(handle_generic_connection(SocketStream::from_udp_stream(
                recv,
            )));
            send
        };

        send.send(Bytes::copy_from_slice(&buffer[..size])).await;
    }

    Ok(())
}

#[pin_project::pin_project(project = EnumProj)]
enum StreamType {
    Tcp(#[pin] TcpStream),
    Udp(#[pin] Receiver<Bytes>),
}

#[pin_project::pin_project]
struct SocketStream {
    #[pin]
    inner: StreamType,

    temp_buffer: [u8; 4096],
}

impl SocketStream {
    pub fn from_tcp_stream(stream: TcpStream) -> Self {
        Self {
            inner: StreamType::Tcp(stream),
            temp_buffer: [0u8; 4096],
        }
    }

    pub fn from_udp_stream(stream: Receiver<Bytes>) -> Self {
        Self {
            inner: StreamType::Udp(stream),
            temp_buffer: [0u8; 4096],
        }
    }
}



impl Stream for SocketStream {
    type Item = Bytes;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let project = self.project();

        match project.inner.project() {
            EnumProj::Tcp(stream) => match stream.poll_read(cx, project.temp_buffer) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Ok(len)) => {
                    Poll::Ready(Some(Bytes::copy_from_slice(&project.temp_buffer[..len])))
                }
                Poll::Ready(Err(_)) => Poll::Ready(None),
            },
            EnumProj::Udp(stream) => stream.poll_next(cx),
        }
    }
}


#[pin_project::pin_project]
struct AppendText<S> {
    #[pin]
    inner: S
}

impl<S> AppendText<S> {
    pub fn wrap(inner: S) -> Self {
        Self { inner }
    }
}

impl<S> Stream for AppendText<S> 
where
    S: Stream<Item = Bytes> + Unpin
{
    type Item = Bytes;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.project().inner.poll_next(cx) {
            Poll::Ready(Some(buf)) => {
                let mut newbuf = BytesMut::new();
                newbuf.extend_from_slice(b"[hey!!] ");
                newbuf.extend_from_slice(&buf);

                Poll::Ready(Some(newbuf.freeze()))
            }

            v => v,
        }
    }
}

async fn handle_generic_connection<S>(mut stream: S) -> anyhow::Result<()>
where
    S: Stream<Item = Bytes> + Unpin,
{
    let mut stream = AppendText::wrap(stream);
    while let Some(buffer) = stream.next().await {
        let data = std::str::from_utf8(&buffer).unwrap();

        print!("recv,,: {}", data);
    }

    Ok(())
}
