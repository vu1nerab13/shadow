use super::{socks5, ProxyType};
use crate::network::ServerObj;
use log::{info, warn};
use remoc::rch;
use shadow_common::{error::ShadowError, transfer, CallResult};
use std::{net::SocketAddr, str::FromStr, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::RwLock,
};
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

    if let Some(_) = server_obj.read().await.proxies.get(&listen_addr) {
        return super::super::success();
    }

    info!("{}://{} proxy started", r#type, listen_addr);

    let so = server_obj.clone();
    let listener = TcpListener::bind(listen_addr).await?;
    let task = tokio::spawn(async move {
        let ret = accept_connection(so.clone(), listener, r#type, user, password).await;

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
    r#type: ProxyType,
    user: String,
    password: String,
) -> CallResult<()> {
    loop {
        let (stream, addr) = match listener.accept().await {
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

        let so = server_obj.clone();
        let user = user.clone();
        let password = password.clone();
        tokio::spawn(async move {
            if let Err(e) = process_connection(so.clone(), r#type, user, password, stream).await {
                warn!(
                    "{}: proxy {} exited unexpectedly, message: {}",
                    so.read().await.get_ip(),
                    addr,
                    e.to_string()
                )
            }
        });
    }
}

async fn process_connection(
    server_obj: Arc<RwLock<ServerObj>>,
    r#type: ProxyType,
    user: String,
    password: String,
    mut stream: TcpStream,
) -> CallResult<()> {
    let handler = match r#type {
        ProxyType::Socks5 => socks5::parse,
    };

    let target_addr = handler(&mut stream, user, password).await?;

    let (server_tx, client_rx) = rch::bin::channel();
    let (client_tx, server_rx) = rch::bin::channel();

    // Send the channels to client
    server_obj
        .read()
        .await
        .proxy(target_addr, client_tx, client_rx)
        .await?;

    transfer(
        server_tx.into_inner().await?,
        server_rx.into_inner().await?,
        stream,
    )
    .await;

    Ok(())
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
