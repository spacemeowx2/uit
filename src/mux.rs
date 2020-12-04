use futures::{
    ready,
    sink::{Sink, SinkExt},
};
use std::collections::VecDeque;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

pub struct SinkDemux<S> {
    index: usize,
    sink: Vec<S>,
    dirty: VecDeque<usize>,
}

impl<S> SinkDemux<S> {
    pub fn new() -> Self {
        Self {
            index: 0,
            sink: vec![],
            dirty: VecDeque::new(),
        }
    }
    fn get_pinned(&mut self) -> Pin<&mut S>
    where
        S: Unpin,
    {
        Pin::new(&mut self.sink[self.index])
    }
}

impl<S, T> Sink<T> for SinkDemux<S>
where
    S: Sink<T> + Unpin,
{
    type Error = S::Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Sink::poll_ready(self.get_pinned(), cx)
    }

    fn start_send(mut self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        let i = self.index;
        self.dirty.push_back(i);
        Sink::start_send(self.get_pinned(), item)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        while let Some(i) = self.dirty.get(0).map(|i| *i) {
            ready!(self.sink[i].poll_flush_unpin(cx))?;
            self.dirty.pop_front();
        }

        self.index += 1;
        if self.index >= self.sink.len() {
            self.index %= self.sink.len();
        }
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        todo!()
    }
}
