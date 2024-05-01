use crate::misc;
use anyhow::Result as AppResult;
use log::info;
use remoc::{codec, prelude::*};
use shadow_common::{
    client as sc,
    server::{self as ss, Server},
    ObjectType, RtcResult,
};
use std::{net::Ipv4Addr, sync::Arc};
use sysinfo::{Components, Disks, Networks, System};
use tokio::{net::TcpStream, sync::RwLock, task::yield_now};
use uuid::Uuid;

#[derive(Debug)]
pub struct ClientCfg {
    version: String,
}

impl Default for ClientCfg {
    fn default() -> Self {
        let version = misc::get_version();

        Self { version }
    }
}

#[derive(Default)]
pub struct ClientObj {
    cfg: ClientCfg,
}

#[rtc::async_trait]
impl sc::Client for ClientObj {
    async fn handshake(&self) -> RtcResult<sc::Handshake> {
        Ok(sc::Handshake {
            message: format!("{:#?}", self.cfg),
        })
    }

    async fn get_system_info(&self) -> RtcResult<sc::SystemInfo> {
        let mut system = System::new_all();
        system.refresh_all();

        let system_name = System::name().unwrap_or_default();
        let kernel_version = System::kernel_version().unwrap_or_default();
        let os_version = System::os_version().unwrap_or_default();
        let host_name = System::host_name().unwrap_or_default();

        let system = format!("{:?}", system);
        let disks = format!("{:?}", Disks::new_with_refreshed_list());
        let networks = format!("{:?}", Networks::new_with_refreshed_list());
        let components = format!("{:?}", Components::new_with_refreshed_list());

        Ok(sc::SystemInfo {
            system_name,
            kernel_version,
            os_version,
            host_name,
            components,
            disks,
            networks,
            system,
        })
    }
}

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
            ObjectType::Uuid(_) => unreachable!(),
        },
        None => todo!(),
    }
}

async fn get_uuid(rx: &mut rch::base::Receiver<ObjectType>) -> AppResult<Uuid> {
    match rx.recv().await? {
        Some(s) => match s {
            ObjectType::ClientClient(_) => unreachable!(),
            ObjectType::ServerClient(_) => unreachable!(),
            ObjectType::Uuid(uuid) => Ok(uuid),
        },
        None => todo!(),
    }
}

async fn handle_connection(client: ss::ServerClient<codec::Bincode>, uuid: Uuid) -> AppResult<()> {
    let handshake = client.handshake(uuid).await?;
    info!("server message: {}", handshake.message);

    loop {
        yield_now().await;
    }
}

pub async fn run() -> AppResult<()> {
    let client_client = run_server()?;
    let (mut tx, mut rx) = connect_server().await?;

    send_client(&mut tx, client_client).await?;

    let server_client = get_client(&mut rx).await?;
    let uuid = get_uuid(&mut rx).await?;

    handle_connection(server_client, uuid).await?;

    Ok(())
}
