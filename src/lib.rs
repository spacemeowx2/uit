use structopt::StructOpt;
use futures::{io::{AsyncRead, AsyncWrite}, stream::{Stream, TryStreamExt}, sink::Sink};
use std::{io, marker::PhantomData, net::SocketAddr, pin::Pin};
use spawn::with_spawn;
use traits::*;

mod integrations;
mod spawn;
mod traits;

#[derive(StructOpt)]
struct ServerOpt {

}

pub struct Server<RT>
where
    RT: AsyncRuntime,
{
    listener: RT::TcpListener,
    _stream: PhantomData<RT>,
}

impl<RT> Server<RT>
where
    RT: AsyncRuntime,
{
    async fn new(addr: SocketAddr) -> io::Result<Self> {
        let listener = RT::bind_tcp(addr).await?;
        Ok(Self {
            listener,
            _stream: PhantomData,
        })
    }
    async fn handle(_s: RT::TcpStream)
    {
        log::trace!("Connection end");
    }
    async fn run(mut self) -> io::Result<()> {
        with_spawn(|s| async move {
            while let Some(stream) = self.listener.try_next().await? {
                s.spawn(Self::handle(stream));
            }
            Ok(())
        }).await
    }
}

pub async fn server_main() -> io::Result<()> {
    use tokio::{select, signal::ctrl_c};

    let server = Server::<integrations::TokioRuntime>::new("0.0.0.0:12342".parse().unwrap()).await?;

    select! {
        r = ctrl_c() => r?,
        r = server.run() => r?,
    };

    Ok(())
}

pub async fn client_main() -> io::Result<()> {
    todo!()
}
