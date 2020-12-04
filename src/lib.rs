use anyhow::Result;
use dashmap::DashMap;
use futures::channel::mpsc;
use futures::prelude::*;
use futures::{
    future::{select, Either},
    stream::FuturesUnordered,
};
use protocol::{FrameCodec, Framed, FramedStream, Message, MAGIC};
use std::{io, pin::Pin};
use structopt::StructOpt;
use tokio::{
    io::BufStream,
    net::{TcpListener, TcpStream},
};
use uuid::Uuid;

mod mux;
mod protocol;

type BoxStream = Pin<Box<dyn FramedStream + Send + Sync + 'static>>;

enum FutureResult {
    None,
}

#[derive(StructOpt)]
struct ServerOpt {}

struct Server {
    context: DashMap<Uuid, ServerContext>,
}
struct ServerContext {
    sender: mpsc::UnboundedSender<BoxStream>,
    // streams: Vec<BoxStream>,
}

impl ServerContext {
    fn new(sender: mpsc::UnboundedSender<BoxStream>) -> Self {
        Self { sender }
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
    async fn handle(&self, s: TcpStream) -> io::Result<FutureResult> {
        let from_addr = s.peer_addr()?;
        s.set_nodelay(true)?;

        let mut s = Framed::new(BufStream::new(s), FrameCodec);

        let uuid = match s.try_next().await? {
            Some(Message::Handshake { magic: MAGIC, uuid }) => uuid,
            Some(Message::Handshake { magic, .. }) => {
                log::trace!(
                    "Failed to handshake from client: {}, peer magic: {:x?}",
                    from_addr,
                    magic
                );
                return Err(err());
            }
            _ => {
                log::trace!("Failed to handshake from client: {}", from_addr);
                return Err(err());
            }
        };

        let uuid = match uuid {
            None => {
                let uuid = Uuid::new_v4();
                let (sender, recver) = mpsc::unbounded();
                self.context.insert(uuid, ServerContext::new(sender));
                s.send(Message::Handshake {
                    magic: MAGIC,
                    uuid: Some(uuid),
                })
                .await?;
                uuid
            }
            Some(uuid) => {
                if !self.context.contains_key(&uuid) {
                    return Err(err());
                }
                uuid
            }
        };

        match self.context.get_mut(&uuid) {
            Some(mut ctx) => {
                // TODO: handle error
                ctx.value_mut().sender.send(Box::pin(s)).await.unwrap();
            }
            None => {}
        }

        log::trace!("Connection end");
        Ok(FutureResult::None)
    }
    async fn run(self, mut listener: TcpListener) -> io::Result<()> {
        let mut tasks = FuturesUnordered::new();
        loop {
            let stream = if tasks.len() > 0 {
                match select(listener.try_next(), tasks.try_next()).await {
                    Either::Left((Ok(Some(result)), _)) => result,
                    Either::Right((result, _)) => {
                        continue;
                    }
                    _ => break,
                }
            } else {
                match listener.try_next().await? {
                    Some(result) => result,
                    _ => break,
                }
            };
            log::trace!("Incoming tcp stream");
            tasks.push(self.handle(stream).boxed())
        }
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
