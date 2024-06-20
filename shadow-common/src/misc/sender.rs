use bytes::Bytes;
use chmux::{SendError, Sender};
use futures::{
    future::BoxFuture,
    ready,
    sink::Sink,
    task::{Context, Poll},
    FutureExt,
};
use remoc::prelude::*;
use std::{pin::Pin, sync::Arc};
use tokio::sync::Mutex;

/// A sink sending byte data over a channel.
pub struct SenderSink {
    sender: Option<Arc<Mutex<Sender>>>,
    send_fut: Option<BoxFuture<'static, Result<(), SendError>>>,
}

impl SenderSink {
    pub fn new(sender: Sender) -> Self {
        Self {
            sender: Some(Arc::new(Mutex::new(sender))),
            send_fut: None,
        }
    }

    pub async fn send(sender: Arc<Mutex<Sender>>, data: Bytes) -> Result<(), SendError> {
        let mut sender = sender.lock().await;
        sender.send(data).await
    }

    pub fn start_send(&mut self, data: Bytes) -> Result<(), SendError> {
        if self.send_fut.is_some() {
            panic!("sink is not ready for sending");
        }

        match self.sender.clone() {
            Some(sender) => {
                self.send_fut = Some(Self::send(sender, data).boxed());
                Ok(())
            }
            None => panic!("start_send after sink has been closed"),
        }
    }

    pub fn poll_send(&mut self, cx: &mut Context) -> Poll<Result<(), SendError>> {
        match &mut self.send_fut {
            Some(fut) => {
                let res = ready!(fut.as_mut().poll(cx));
                self.send_fut = None;
                Poll::Ready(res)
            }
            None => Poll::Ready(Ok(())),
        }
    }

    pub fn close(&mut self) {
        self.sender = None;
    }
}

impl Sink<&[u8]> for SenderSink {
    type Error = SendError;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Pin::into_inner(self).poll_send(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: &[u8]) -> Result<(), Self::Error> {
        let item = item.to_owned();
        Pin::into_inner(self).start_send(item.into())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Pin::into_inner(self).poll_send(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        ready!(Pin::into_inner(self.as_mut()).poll_send(cx))?;
        Pin::into_inner(self).close();
        Poll::Ready(Ok(()))
    }
}
