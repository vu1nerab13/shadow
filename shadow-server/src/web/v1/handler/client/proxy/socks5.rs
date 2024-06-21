use crate::network::ServerObj;
use log::warn;
use remoc::rch;
use shadow_common::{error::ShadowError, transfer};
use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, sync::RwLock};
use warp::reply::Reply;

pub async fn open(
    server_obj: Arc<RwLock<ServerObj>>,
    listen_addr: SocketAddr,
) -> Result<Box<dyn Reply>, ShadowError> {
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
        let (stream, _addr) = match listener.accept().await {
            Ok(r) => r,
            Err(e) => {
                warn!(
                    "{}: error in accepting incoming proxy request, message: {}",
                    server_obj.read().await.get_ip(),
                    e.to_string()
                );

                continue;
            }
        };

        let (server_tx, client_rx) = rch::bin::channel();
        let (client_tx, server_rx) = rch::bin::channel();

        // Send the channels to client
        match server_obj
            .read()
            .await
            .proxy("127.0.0.1:8001".parse()?, client_tx, client_rx)
            .await
        {
            Ok(_) => {}
            Err(e) => {
                warn!(
                    "{}: error in sending channels to client, message: {}",
                    server_obj.read().await.get_ip(),
                    e.to_string()
                );

                continue;
            }
        }

        tokio::spawn(transfer(
            server_tx.into_inner().await?,
            server_rx.into_inner().await?,
            stream,
        ));
    }
}

pub async fn close(
    server_obj: Arc<RwLock<ServerObj>>,
    listen_addr: SocketAddr,
) -> Result<Box<dyn Reply>, ShadowError> {
    let mut server_obj = server_obj.write().await;

    server_obj
        .proxies
        .get(&listen_addr)
        .ok_or(ShadowError::ParamInvalid(
            "no existing proxy running".into(),
        ))?
        .abort();

    server_obj.proxies.remove(&listen_addr);

    super::super::success()
}
