use crate::network::server::ClientObj;
use anyhow::Result as AppResult;
use log::info;
use remoc::{codec, prelude::*};
use rustls::client::danger::ServerCertVerifier;
use rustls_pki_types::ServerName;
use shadow_common::{
    client as sc,
    server::{self as ss, Server},
    ObjectType,
};
use std::{io::Cursor, net::SocketAddr, sync::Arc, time::Duration};
use tokio::{
    fs::File,
    io::{self, AsyncReadExt},
    net::TcpStream,
    sync::RwLock,
    time,
};
use tokio_rustls::{
    rustls::{self, RootCertStore},
    TlsConnector,
};

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
    let socket = TcpStream::connect(addr).await?;
    let socket = connector
        .connect(ServerName::IpAddress(addr.ip().into()), socket)
        .await?;
    let (socket_rx, socket_tx) = io::split(socket);
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
        None => unreachable!(),
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

async fn add_to_ca() -> AppResult<RootCertStore> {
    let mut root_cert_store = rustls::RootCertStore::empty();
    let mut content = Vec::new();
    File::open("/Users/mitsuha/shadow/certs/shadow_ca.crt")
        .await?
        .read_to_end(&mut content)
        .await?;

    for cert in rustls_pemfile::certs(&mut Cursor::new(content)) {
        root_cert_store.add(cert?)?;
    }

    Ok(root_cert_store)
}

#[derive(Debug)]
struct NoCertificateVerification {}

impl Default for NoCertificateVerification {
    fn default() -> Self {
        Self {}
    }
}

impl ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls_pki_types::CertificateDer<'_>,
        _intermediates: &[rustls_pki_types::CertificateDer<'_>],
        _server_name: &rustls_pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls_pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls_pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls_pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        rustls::crypto::aws_lc_rs::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

pub async fn run(cfg: Config) -> AppResult<()> {
    let root_cert_store = add_to_ca().await?;
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
