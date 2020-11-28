use futures::{io::{AsyncRead, AsyncWrite}, Stream, Sink, future::BoxFuture};
use std::{io, net::SocketAddr};

pub trait AsyncListener<TcpStream>: Stream<Item = io::Result<TcpStream>> + Send + Unpin + 'static {}
impl<T, TcpStream> AsyncListener<TcpStream> for T
where
    T: Stream<Item = io::Result<TcpStream>> + Send + Unpin + 'static,
{}

pub trait AsyncStream: AsyncRead + AsyncWrite + Send + 'static {}
impl<T> AsyncStream for T
where
    T: AsyncRead + AsyncWrite + Send + 'static,
{}

pub type UdpItem = (Vec<u8>, SocketAddr);
pub trait AsyncUdp: Stream<Item=UdpItem> + Sink<UdpItem, Error=io::Error> + Send + 'static {}
impl<T> AsyncUdp for T
where
    T: Stream<Item=UdpItem> + Sink<UdpItem, Error=io::Error> + Send + 'static,
{}

pub trait AsyncRuntime: Send + 'static {
    type TcpStream: AsyncStream;
    type TcpListener: AsyncListener<Self::TcpStream>;
    type UdpSocket: AsyncUdp;

    fn bind_udp() -> BoxFuture<'static, io::Result<Self::UdpSocket>>;
    fn bind_tcp(addr: SocketAddr) -> BoxFuture<'static, io::Result<Self::TcpListener>>;
}
