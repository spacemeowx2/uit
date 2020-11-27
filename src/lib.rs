use structopt::StructOpt;
use futures::{io::{AsyncRead, AsyncWrite}, stream::{Stream, StreamExt, FuturesUnordered}, future::{select, Either}};
use std::io;

pub trait AsyncStream: AsyncRead + AsyncWrite {

}
impl<T> AsyncStream for T
where
    T: AsyncRead + AsyncWrite,
{}

#[derive(StructOpt)]
struct ServerOpt {

}

struct Server<TcpStream, TcpListener>
where
    TcpListener: Stream<Item = io::Result<TcpStream>> + Unpin,
    TcpStream: AsyncStream,
{
    listener: TcpListener
}

impl<TcpStream, TcpListener> Server<TcpStream, TcpListener>
where
    TcpListener: Stream<Item = io::Result<TcpStream>> + Unpin,
    TcpStream: AsyncStream,
{

    async fn handle(_s: TcpStream)
    {
        log::trace!("Connection end");
    }
    async fn run(mut self) -> io::Result<()> {
        let mut tasks = FuturesUnordered::new();
        loop {
            let result = if tasks.len() > 0 {
                match select(self.listener.next(), tasks.next()).await {
                    Either::Left((Some(result), _)) => result,
                    Either::Right(_) => continue,
                    _ => break,
                }
            } else {
                match self.listener.next().await {
                    Some(result) => result,
                    _ => break,
                }
            };
            log::trace!("Incoming tcp stream");
            let stream = result?;
            tasks.push(Self::handle(stream))
        };
        Ok(())
    }
}

pub async fn server_main<TcpStream, TcpListener>(listener: TcpListener) -> io::Result<()>
where
    TcpListener: Stream<Item = io::Result<TcpStream>> + Unpin,
    TcpStream: AsyncStream,
{
    Server {
        listener
    }.run().await
}

pub async fn client_main() -> io::Result<()> {
    todo!()
}
