mod tokio {
    use crate::{Server, AsyncRuntime, UdpItem, AsyncUdp};
    use tokio_util::compat::*;
    use tokio::net::{TcpListener, TcpStream};
    use futures::{stream::{BoxStream, StreamExt, Stream}, sink::{Sink}, future::{FutureExt, BoxFuture}};
    use std::{io, pin::Pin};

    pub struct TokioRuntime {}

    impl AsyncRuntime for TokioRuntime {
        type TcpStream = Compat<TcpStream>;
        type TcpListener = BoxStream<'static, io::Result<Self::TcpStream>>;
        type UdpSocket = Pin<Box<dyn AsyncUdp + Send + 'static>>;

        fn bind_udp() -> BoxFuture<'static, io::Result<Self::UdpSocket>> {
            todo!()
        }

        fn bind_tcp(addr: std::net::SocketAddr) -> BoxFuture<'static, io::Result<Self::TcpListener>> {
            let fut = async move {
                let listener = TcpListener::bind(addr).await?;
                let listener = listener.map(|s| s.map(|s| {
                    s.set_nodelay(true).ok();
                    s.compat()
                }));
                Ok(listener.boxed())
            };
            Box::pin(fut.boxed())
        }
    }
}
pub use self::tokio::TokioRuntime;
