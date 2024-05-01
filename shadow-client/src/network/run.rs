use crate::network::server::ClientObj;
use anyhow::Result as AppResult;
use log::info;
use remoc::{codec, prelude::*};
use shadow_common::{
    client as sc,
    server::{self as ss, Server},
    ObjectType,
};
use std::{net::Ipv4Addr, sync::Arc};
use tokio::{net::TcpStream, sync::RwLock, task::yield_now};

fn run_server() -> AppResult<sc::ClientClient<codec::Bincode>> {
    let client_obj = Arc::new(RwLock::new(ClientObj::default()));
    let (server, client_client) =
        sc::ClientServerSharedMut::<_, codec::Bincode>::new(client_obj, 1);

    tokio::spawn(server.serve(true));

    Ok(client_client)
}

async fn send_client(
    tx: &mut rch::base::Sender<ObjectType>,
    client_client: sc::ClientClient<codec::Bincode>,
) -> AppResult<()> {
    tx.send(ObjectType::ClientClient(client_client)).await?;

    Ok(())
}

async fn connect_server() -> AppResult<(
    rch::base::Sender<ObjectType>,
    rch::base::Receiver<ObjectType>,
)> {
    let socket = TcpStream::connect((Ipv4Addr::LOCALHOST, 1244)).await?;
    let (socket_rx, socket_tx) = socket.into_split();
    let (conn, tx, rx): (
        _,
        rch::base::Sender<ObjectType>,
        rch::base::Receiver<ObjectType>,
    ) = remoc::Connect::io(remoc::Cfg::default(), socket_rx, socket_tx).await?;
    tokio::spawn(conn);

    Ok((tx, rx))
}

async fn get_client(
    rx: &mut rch::base::Receiver<ObjectType>,
) -> AppResult<ss::ServerClient<codec::Bincode>> {
    match rx.recv().await? {
        Some(s) => match s {
            ObjectType::ClientClient(_) => unreachable!(),
            ObjectType::ServerClient(server_client) => Ok(server_client),
        },
        None => todo!(),
    }
}

async fn handle_connection(client: ss::ServerClient<codec::Bincode>) -> AppResult<()> {
    let handshake = client.handshake().await?;
    info!("server message: {}", handshake.message);

    loop {
        match client.is_closed() {
            true => break,
            false => yield_now().await,
        }
    }

    Ok(())
}

pub async fn run() -> AppResult<()> {
    let client_client = run_server()?;
    let (mut tx, mut rx) = connect_server().await?;

    send_client(&mut tx, client_client).await?;

    let server_client = get_client(&mut rx).await?;

    handle_connection(server_client).await?;

    Ok(())
}
