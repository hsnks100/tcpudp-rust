
use async_std::net::{self, TcpListener, TcpStream};
pub enum StreamType<'a> {
    Tcp(&'a TcpStream),
    Udp(&'a async_std::net::UdpSocket),
}