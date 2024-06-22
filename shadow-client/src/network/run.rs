use crate::network::{
    server::ClientObj,
    tls::{self, NoCertificateVerification},
};
use anyhow::Result as AppResult;
use log::info;
use remoc::{codec, prelude::*};
use rustls_pki_types::ServerName;
use shadow_common::{
    client as sc,
    error::ShadowError,
    server::{self as ss, Server},
    ObjectType,
};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{io, net::TcpStream, sync::RwLock, time};
use tokio_rustls::{rustls, TlsConnector};

pub struct Config {
    addr: SocketAddr,
}

impl Config {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }
}

fn run_server() -> AppResult<(Arc<RwLock<ClientObj>>, sc::ClientClient<codec::Bincode>)> {
    let client_obj = Arc::new(RwLock::new(ClientObj::new()));
    let (server, client_client) =
        sc::ClientServerSharedMut::<_, codec::Bincode>::new(client_obj.clone(), 1);

    tokio::spawn(server.serve(true));

    Ok((client_obj, client_client))
}

async fn send_client(
    tx: &mut rch::base::Sender<ObjectType>,
    client_client: sc::ClientClient<codec::Bincode>,
) -> AppResult<()> {
    tx.send(ObjectType::ClientClient(client_client)).await?;

    Ok(())
}

async fn connect_server(
    addr: SocketAddr,
    connector: TlsConnector,
) -> AppResult<(
    rch::base::Sender<ObjectType>,
    rch::base::Receiver<ObjectType>,
)> {
    let stream = TcpStream::connect(addr).await?;
    let stream = connector
        .connect(ServerName::IpAddress(addr.ip().into()), stream)
        .await?;
    let (socket_rx, socket_tx) = io::split(stream);
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
            ObjectType::ClientClient(_) => Err(Box::new(ShadowError::ParamInvalid(
                "expect server, but receive client".into(),
            ))
            .into()),
            ObjectType::ServerClient(server_client) => Ok(server_client),
        },
        None => Err(Box::new(ShadowError::ParamInvalid("can not receive server".into())).into()),
    }
}

async fn handle_connection(client: Arc<RwLock<ss::ServerClient<codec::Bincode>>>) -> AppResult<()> {
    let client = client.read().await;

    let handshake = client.handshake().await?;
    info!("server message: {}", handshake.message);

    loop {
        match client.is_closed() {
            true => break,
            false => time::sleep(Duration::from_secs(10)).await,
        }
    }

    Ok(())
}

pub async fn run(cfg: Config) -> AppResult<()> {
    let root_cert_store = tls::add_to_ca().await?;
    let mut config = rustls::ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();
    config
        .dangerous()
        .set_certificate_verifier(Arc::new(NoCertificateVerification::default()));
    let connector = TlsConnector::from(Arc::new(config));
    let (client_obj, client_client) = run_server()?;
    let (mut tx, mut rx) = connect_server(cfg.addr, connector).await?;

    send_client(&mut tx, client_client).await?;

    let server_client = Arc::new(RwLock::new(get_client(&mut rx).await?));
    client_obj.write().await.client = Some(server_client.clone());

    handle_connection(server_client).await?;

    Ok(())
}
