use serde_derive::{Serialize, Deserialize};
use std::io;
pub use tokio_util::codec::{Decoder, Encoder, Framed};
use bytes::{BytesMut, Buf, BufMut};
use uuid::Uuid;
use futures::{Stream, Sink};

pub const MAGIC: [u8; 4] = [0x11, 0x66, 0x33, 0x22];

#[derive(Serialize, Deserialize)]
pub enum Message {
    Handshake{
        magic: [u8; 4],
        uuid: Option<Uuid>,
    },
}

pub struct FrameCodec;

impl Decoder for FrameCodec {
    type Item = Message;

    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 2 {
            src.reserve(2);
            return Ok(None);
        }
        let size = src.get_u16() as usize;
        if src.remaining() < size {
            src.reserve(2 + size);
            return Ok(None);
        }

        let body = src.take(size).reader();
        let msg: Message = bincode::deserialize_from(body).map_err(|_| io::ErrorKind::InvalidData)?;
        Ok(Some(msg))
    }
}

impl Encoder<Message> for FrameCodec {
    type Error = io::Error;

    fn encode(&mut self, item: Message, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let len = bincode::serialized_size(&item).map_err(|_| io::ErrorKind::InvalidData)?;
        dst.put_u16(len as u16);
        bincode::serialize_into(dst.writer(), &item).map_err(|_| io::ErrorKind::InvalidData)?;

        Ok(())
    }
}

pub trait FramedStream: Stream<Item=io::Result<Message>> + Sink<Message, Error=io::Error> {}
impl<T> FramedStream for T
where
    T: Stream<Item=io::Result<Message>> + Sink<Message, Error=io::Error>,
{}