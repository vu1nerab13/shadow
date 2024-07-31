use super::{socks5, ProxyType};
use crate::network::ServerObj;
use log::{info, warn};
use remoc::rch;
use shadow_common::{error::ShadowError, misc::sender, CallResult};
use std::{net::SocketAddr, str::FromStr, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    select,
    sync::{oneshot, RwLock},
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
    let (signal_tx, signal_rx) = oneshot::channel();
    let _ = tokio::spawn(async move {
        select! {
            ret = accept_connection(so.clone(), listener, r#type, user, password) => {
                match ret {
                    Ok(_) => info!("{}://{} proxy stopped", r#type, listen_addr),
                    Err(e) => warn!(
                        "{}://{} proxy stopped unexpectedly, message: {}",
                        r#type,
                        listen_addr,
                        e.to_string()
                    ),
                };
            }

            _ = signal_rx => info!("{}://{} proxy stopped", r#type, listen_addr)
        }

        so.write().await.proxies.remove(&listen_addr);
    });

    server_obj
        .write()
        .await
        .proxies
        .insert(listen_addr, Some(signal_tx));

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
                    server_obj.read().await.ip(),
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
                    so.read().await.ip(),
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
    // Get specific parser on demand
    let handler = match r#type {
        ProxyType::Socks5 => socks5::parse,
    };

    // Parse the request
    let target_addr = handler(&mut stream, user, password).await?;

    // Initialize channels for data exchange
    let (server_tx, client_rx) = rch::bin::channel();
    let (client_tx, server_rx) = rch::bin::channel();

    // Send the channels to client
    let signal_rx = server_obj
        .read()
        .await
        .proxy(target_addr, client_tx, client_rx)
        .await?;

    // Drop unnecessary reference
    drop(server_obj);

    select! {
        // If server-side disconnect
        _ = sender::transfer(
            server_tx.into_inner().await?,
            server_rx.into_inner().await?,
            stream,
        ) => {}
        // If client-side disconnect
        _ = signal_rx => {}
    }

    Ok(())
}

pub async fn close(
    server_obj: Arc<RwLock<ServerObj>>,
    listen_addr: SocketAddr,
) -> CallResult<Box<dyn Reply>> {
    let mut server_obj = server_obj.write().await;

    // Get current proxy instance, if exist, issue stop command
    if let Some(tx) = server_obj
        .proxies
        .get_mut(&listen_addr)
        .ok_or(ShadowError::ParamInvalid(
            "no existing proxy running".into(),
        ))?
        .take()
    {
        tx.send(true).map_err(|_| ShadowError::StopFailed)?;
    }

    // Remove instance from the hashmap
    server_obj.proxies.remove(&listen_addr);

    super::super::success()
}
