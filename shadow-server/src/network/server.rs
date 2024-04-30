use crate::misc;
use anyhow::Result as AppResult;
use log::{info, trace};
use remoc::{codec, prelude::*, rch};
use shadow_common::{
    client::{self as sc, Client},
    server as ss, ObjectType, RtcResult,
};
use std::{net::Ipv4Addr, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::RwLock,
};

#[derive(Debug)]
pub struct ServerCfg {
    version: String,
}

impl Default for ServerCfg {
    fn default() -> Self {
        let version = misc::get_version();

        Self { version }
    }
}

#[derive(Default)]
pub struct ServerObj {
    cfg: ServerCfg,
}

#[rtc::async_trait]
impl ss::Server for ServerObj {
    async fn handshake(&self) -> RtcResult<ss::Handshake> {
        Ok(ss::Handshake {
            message: format!("{:#?}", self.cfg),
        })
    }
}

fn run_server(server_obj: Arc<RwLock<ServerObj>>) -> AppResult<ss::ServerClient<codec::Bincode>> {
    let (server, server_client) =
        ss::ServerServerSharedMut::<_, codec::Bincode>::new(server_obj, 1);

    tokio::spawn(server.serve(true));
    Ok(server_client)
}

async fn send_client(
    tx: &mut rch::base::Sender<ObjectType>,
    server_client: ss::ServerClient<codec::Bincode>,
) -> AppResult<()> {
    tx.send(ObjectType::ServerClient(server_client)).await?;

    Ok(())
}

async fn connect_client(
    socket: TcpStream,
) -> AppResult<(
    rch::base::Sender<ObjectType>,
    rch::base::Receiver<ObjectType>,
)> {
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
) -> AppResult<sc::ClientClient<codec::Bincode>> {
    match rx.recv().await? {
        Some(s) => match s {
            ObjectType::ClientClient(client_client) => Ok(client_client),
            ObjectType::ServerClient(_) => unreachable!(),
        },
        None => todo!(),
    }
}

async fn handle_connection(client: sc::ClientClient<codec::Bincode>) -> AppResult<()> {
    let handshake = client.handshake().await?;
    info!("client message: {}", handshake.message);

    let sys_info = client.get_system_info().await?;
    info!("client info: {:#?}", sys_info);

    Ok(())
}

pub async fn run() -> AppResult<()> {
    let server_obj = Arc::new(RwLock::new(ServerObj::default()));

    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 1244)).await?;

    loop {
        let server_obj = server_obj.clone();
        let (socket, addr) = listener.accept().await?;

        tokio::spawn(async move {
            let server_client = run_server(server_obj)?;
            let (mut tx, mut rx) = connect_client(socket).await?;

            send_client(&mut tx, server_client).await?;

            let client_client = get_client(&mut rx).await?;

            trace!("{}: connected", addr);
            handle_connection(client_client).await?;

            Ok::<(), anyhow::Error>(())
        });
    }
}
