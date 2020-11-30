use structopt::StructOpt;
use std::{io, pin::Pin};
use tokio::{io::BufStream, net::{TcpListener, TcpStream}};
use anyhow::Result;
use protocol::{FramedStream, FrameCodec, Message, MAGIC, Framed};
use futures::prelude::*;
use futures::{stream::{FuturesUnordered}, future::{select, Either}};
use dashmap::DashMap;
use uuid::Uuid;

mod protocol;
mod mux;

type BoxStream = Pin<Box<dyn FramedStream + Send + Sync + 'static>>;

#[derive(StructOpt)]
struct ServerOpt {

}

struct Server {
    context: DashMap::<Uuid, ServerContext>,
}
struct ServerContext {
    streams: Vec<BoxStream>,
}

impl ServerContext {
    fn new() -> Self {
        Self {
            streams: vec![],
        }
    }
}

fn err() -> io::Error {
    io::Error::new(io::ErrorKind::Other, "Protocol error".to_string())
}

impl Server {
    fn new() -> Server {
        Server {
            context: DashMap::new(),
        }
    }
    async fn handle(&self, s: TcpStream) -> io::Result<()>
    {
        let from_addr = s.peer_addr()?;
        s.set_nodelay(true)?;

        let mut s = Framed::new(BufStream::new(s), FrameCodec);

        let uuid = match s.try_next().await? {
            Some(Message::Handshake{
                magic: MAGIC,
                uuid,
            }) => {
                uuid
            }
            Some(Message::Handshake{
                magic,
                ..
            }) => {
                log::trace!("Failed to handshake from client: {}, peer magic: {:x?}", from_addr, magic);
                return Err(err())
            }
            _ => {
                log::trace!("Failed to handshake from client: {}", from_addr);
                return Err(err())
            }
        };

        let uuid = match uuid {
            None => {
                let uuid = Uuid::new_v4();
                self.context.insert(uuid, ServerContext::new());
                s.send(Message::Handshake{
                    magic: MAGIC,
                    uuid: Some(uuid),
                }).await?;
                uuid
            },
            Some(uuid) => {
                if !self.context.contains_key(&uuid) {
                    return Err(err())
                }
                uuid
            },
        };

        match self.context.get_mut(&uuid) {
            Some(mut ctx) => {
                ctx.value_mut().streams.push(Box::pin(s));
            },
            None => {},
        }

        log::trace!("Connection end");
        Ok(())
    }
    async fn run(self, mut listener: TcpListener) -> io::Result<()> {
        let mut tasks = FuturesUnordered::new();
        loop {
            let stream = if tasks.len() > 0 {
                match select(listener.try_next(), tasks.try_next()).await {
                    Either::Left((Ok(Some(result)), _)) => result,
                    Either::Right((_result, _)) => continue,
                    _ => break,
                }
            } else {
                match listener.try_next().await? {
                    Some(result) => result,
                    _ => break,
                }
            };
            log::trace!("Incoming tcp stream");
            tasks.push(self.handle(stream))
        };
        Ok(())
    }
}

pub async fn server_main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:12341").await?;

    Server::new().run(listener).await?;

    Ok(())
}

pub async fn client_main() -> io::Result<()> {
    todo!()
}
