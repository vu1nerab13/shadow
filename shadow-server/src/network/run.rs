use crate::network::{server::ServerObj, tls};
use anyhow::Result as AppResult;
use log::info;
use remoc::{chmux::ChMuxError, codec, prelude::*, rch};
use shadow_common::{
    client::{self as sc, Client},
    server as ss, ObjectType,
};
use std::{collections::HashMap, net::SocketAddr, path::Path, sync::Arc};
use tokio::{
    io::{self},
    net::{TcpListener, TcpStream},
    sync::RwLock,
    task::JoinHandle,
};
use tokio_rustls::{rustls, server::TlsStream, TlsAcceptor};

pub struct Config {
    addr: SocketAddr,
}

impl Config {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }
}

fn run_server(
    addr: SocketAddr,
    server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
) -> AppResult<(Arc<RwLock<ServerObj>>, ss::ServerClient<codec::Bincode>)> {
    let server_obj = Arc::new(RwLock::new(ServerObj::new(addr)));
    let (server, server_client) =
        ss::ServerServerSharedMut::<_, codec::Bincode>::new(server_obj.clone(), 1);

    tokio::spawn(async move {
        server.serve(true).await;

        info!("{}: disconnected", addr);
        server_objs.write().await.remove(&addr);
    });

    Ok((server_obj, server_client))
}

async fn send_client(
    tx: &mut rch::base::Sender<ObjectType>,
    server_client: ss::ServerClient<codec::Bincode>,
) -> AppResult<()> {
    tx.send(ObjectType::ServerClient(server_client)).await?;

    Ok(())
}

async fn connect_client(
    socket: TlsStream<TcpStream>,
) -> AppResult<(
    JoinHandle<Result<(), ChMuxError<std::io::Error, std::io::Error>>>,
    rch::base::Sender<ObjectType>,
    rch::base::Receiver<ObjectType>,
)> {
    let (socket_rx, socket_tx) = io::split(socket);
    let (conn, tx, rx): (
        _,
        rch::base::Sender<ObjectType>,
        rch::base::Receiver<ObjectType>,
    ) = remoc::Connect::io(remoc::Cfg::default(), socket_rx, socket_tx).await?;

    Ok((tokio::spawn(conn), tx, rx))
}

async fn get_client(
    rx: &mut rch::base::Receiver<ObjectType>,
) -> AppResult<sc::ClientClient<codec::Bincode>> {
    match rx.recv().await? {
        Some(s) => match s {
            ObjectType::ClientClient(client_client) => Ok(client_client),
            ObjectType::ServerClient(_) => unreachable!(),
        },
        None => unreachable!(),
    }
}

async fn handle_connection(
    addr: SocketAddr,
    server_obj: Arc<RwLock<ServerObj>>,
    client: Arc<RwLock<sc::ClientClient<codec::Bincode>>>,
) -> AppResult<()> {
    let client = client.read().await;

    let handshake = client.handshake().await?;
    info!("{}: message: {}", addr, handshake.message);

    let sys_info = client.get_system_info().await?;
    info!("{}: info: {:#?}", addr, sys_info);
    server_obj.write().await.info = sys_info;

    Ok(())
}

pub async fn run(
    cfg: Config,
    server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
) -> AppResult<()> {
    let certs = tls::load_certs(Path::new("certs/shadow_ca.crt")).await?;
    let key = tls::load_keys(Path::new("certs/rsa_4096_pri.key")).await?;
    let tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;
    let acceptor = TlsAcceptor::from(Arc::new(tls_config));
    let listener = TcpListener::bind(cfg.addr).await?;

    loop {
        let server_objs = server_objs.clone();
        let acceptor = acceptor.clone();
        let (stream, addr) = listener.accept().await?;

        tokio::spawn(async move {
            let socket = acceptor.accept(stream).await?;
            let (server_obj, server_client) = run_server(addr, server_objs.clone())?;
            let (task, mut tx, mut rx) = connect_client(socket).await?;

            server_objs.write().await.insert(addr, server_obj.clone());
            send_client(&mut tx, server_client).await?;

            let client_client = Arc::new(RwLock::new(get_client(&mut rx).await?));
            server_obj.write().await.client = Some(client_client.clone());
            server_obj.write().await.task = Some(task);

            info!("{}: connected", addr);
            handle_connection(addr, server_obj, client_client).await?;

            Ok::<(), anyhow::Error>(())
        });
    }
}
