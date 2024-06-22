use crate::{error::ShadowError, CallResult};
use bytes::Bytes;
use chmux::{ReceiverStream, Sender};
use futures::{
    future::BoxFuture,
    ready, select,
    sink::Sink,
    task::{Context, Poll},
    FutureExt,
};
use remoc::prelude::*;
use std::{pin::Pin, sync::Arc};
use tokio::{io, net::TcpStream, sync::Mutex};
use tokio_util::io::{SinkWriter, StreamReader};

/// A sink sending byte data over a channel.
pub struct SenderSink {
    sender: Option<Arc<Mutex<Sender>>>,
    send_fut: Option<BoxFuture<'static, CallResult<()>>>,
}

impl SenderSink {
    pub fn new(sender: Sender) -> Self {
        Self {
            sender: Some(Arc::new(Mutex::new(sender))),
            send_fut: None,
        }
    }

    pub async fn send(sender: Arc<Mutex<Sender>>, data: Bytes) -> CallResult<()> {
        let mut sender = sender.lock().await;
        Ok(sender.send(data).await?)
    }

    pub fn start_send(&mut self, data: Bytes) -> CallResult<()> {
        if self.send_fut.is_some() {
            return Err(ShadowError::ParamInvalid(
                "sink is not ready for sending".into(),
            ));
        }

        match self.sender.clone() {
            Some(sender) => {
                self.send_fut = Some(Self::send(sender, data).boxed());
                Ok(())
            }
            None => {
                return Err(ShadowError::ParamInvalid(
                    "start_send after sink has been closed".into(),
                ))
            }
        }
    }

    pub fn poll_send(&mut self, cx: &mut Context) -> Poll<CallResult<()>> {
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
    type Error = ShadowError;

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

pub async fn transfer(sender: chmux::Sender, receiver: chmux::Receiver, stream: TcpStream) {
    let (mut rx, mut tx) = io::split(stream);
    let mut reader = StreamReader::new(ReceiverStream::new(receiver));
    let mut writer = SinkWriter::new(SenderSink::new(sender));
    let task1 = io::copy(&mut rx, &mut writer).fuse();
    let task2 = io::copy(&mut reader, &mut tx).fuse();

    let mut task1 = Box::pin(task1);
    let mut task2 = Box::pin(task2);

    // I don't want to use the return value
    select! {
        _ = task1 => {}
        _ = task2 => {}
    }
}
