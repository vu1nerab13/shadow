use crate::network::ServerObj;
use remoc::{
    chmux::{self, ReceiverStream},
    rch,
};
use shadow_common::{error::ShadowError, SenderSink};
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    io, join,
    net::{TcpListener, TcpStream},
    sync::RwLock,
};
use tokio_util::io::{SinkWriter, StreamReader};
use warp::reply::Reply;

pub async fn open(
    server_obj: Arc<RwLock<ServerObj>>,
    addr: &String,
    port: u16,
) -> Result<Box<dyn Reply>, ShadowError> {
    let listen_addr: SocketAddr = format!("{}:{}", addr, port).parse()?;

    if let Some(_task) = server_obj.read().await.proxies.get(&listen_addr) {
        return super::super::success();
    }

    let listener = TcpListener::bind(listen_addr).await?;
    let task = tokio::spawn(accept_connection(server_obj.clone(), listener));

    server_obj.write().await.proxies.insert(listen_addr, task);

    super::super::success()
}

async fn accept_connection(
    server_obj: Arc<RwLock<ServerObj>>,
    listener: TcpListener,
) -> Result<(), ShadowError> {
    loop {
        let (stream, _addr) = listener.accept().await?;
        let (server_tx, client_rx) = rch::bin::channel();
        let (client_tx, server_rx) = rch::bin::channel();

        server_obj.read().await.proxy(client_tx, client_rx).await?;

        tokio::spawn(transfer(
            server_tx.into_inner().await?,
            server_rx.into_inner().await?,
            stream,
        ));
    }
}

async fn transfer(sender: chmux::Sender, receiver: chmux::Receiver, stream: TcpStream) {
    let (mut rx, mut tx) = io::split(stream);
    let task1 = tokio::spawn(async move {
        io::copy(&mut rx, &mut SinkWriter::new(SenderSink::new(sender))).await?;

        Ok::<(), anyhow::Error>(())
    });
    let task2 = tokio::spawn(async move {
        io::copy(
            &mut StreamReader::new(ReceiverStream::new(receiver)),
            &mut tx,
        )
        .await?;

        Ok::<(), anyhow::Error>(())
    });

    let _ = join!(task1, task2);
}

pub async fn close(
    server_obj: Arc<RwLock<ServerObj>>,
    addr: &String,
    port: u16,
) -> Result<Box<dyn Reply>, ShadowError> {
    let listen_addr: SocketAddr = format!("{}:{}", addr, port).parse()?;
    let server_obj = server_obj.read().await;

    server_obj
        .proxies
        .get(&listen_addr)
        .ok_or(ShadowError::ParamInvalid(
            "no existing proxy running".into(),
        ))?
        .abort();

    super::super::success()
}
