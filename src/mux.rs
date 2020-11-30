use futures::{Sink, ready};
use std::{task::{Context, Poll}, pin::Pin};

pub struct SinkDemux<S> {
    index: usize,
    sink: Vec<S>,
}

impl<S> SinkDemux<S> {
    fn get_pinned(&mut self) -> Pin<&mut S>
    where
        S: Unpin,
    {
        if self.index >= self.sink.len() {
            self.index %= self.sink.len();
        }
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
        Sink::start_send(self.get_pinned(), item)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let r = ready!(Sink::poll_flush(self.get_pinned(), cx));
        self.index += 1;
        Poll::Ready(r)
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        todo!()
    }
}