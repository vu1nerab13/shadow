use super::{socks5, ProxyType};
use crate::network::ServerObj;
use log::{info, warn};
use remoc::rch;
use shadow_common::{error::ShadowError, transfer, CallResult};
use std::{net::SocketAddr, str::FromStr, sync::Arc};
use tokio::{net::TcpListener, sync::RwLock};
use warp::reply::Reply;

pub async fn open(
    server_obj: Arc<RwLock<ServerObj>>,
    listen_addr: SocketAddr,
    r#type: &String,
    user: String,
    password: String,
) -> CallResult<Box<dyn Reply>> {
    let r#type = ProxyType::from_str(&r#type)
        .map_err(|_| ShadowError::ParamInvalid("unsupported proxy type".into()))?;

    if let Some(_task) = server_obj.read().await.proxies.get(&listen_addr) {
        return super::super::success();
    }

    info!("{}://{} proxy started", r#type, listen_addr);

    let so = server_obj.clone();
    let listener = TcpListener::bind(listen_addr).await?;
    let task = tokio::spawn(async move {
        let ret = accept_connection(so.clone(), listener, &r#type, user, password).await;

        match ret {
            Ok(_) => info!("{}://{} proxy stopped", r#type, listen_addr),
            Err(e) => warn!(
                "{}://{} proxy stopped unexpectedly, message: {}",
                r#type,
                listen_addr,
                e.to_string()
            ),
        };

        so.write().await.proxies.remove(&listen_addr);
    });

    server_obj.write().await.proxies.insert(listen_addr, task);

    super::super::success()
}

async fn accept_connection(
    server_obj: Arc<RwLock<ServerObj>>,
    listener: TcpListener,
    r#type: &ProxyType,
    user: String,
    password: String,
) -> CallResult<()> {
    loop {
        let (mut stream, _addr) = match listener.accept().await {
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

        let handler = match r#type {
            ProxyType::Socks5 => socks5::parse,
        };

        let target_addr = match handler(&mut stream, user.clone(), password.clone()).await {
            Ok(t) => t,
            Err(e) => {
                warn!(
                    "{}: error in parsing protocol, message: {}",
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
            .proxy(target_addr, client_tx, client_rx)
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
) -> CallResult<Box<dyn Reply>> {
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
