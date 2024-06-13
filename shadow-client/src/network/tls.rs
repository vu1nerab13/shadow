use anyhow::Result as AppResult;
use rustls::client::danger::ServerCertVerifier;
use std::io::Cursor;
use tokio::{fs::File, io::AsyncReadExt};
use tokio_rustls::rustls::{self, RootCertStore};

#[derive(Debug)]
pub struct NoCertificateVerification {}

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

pub async fn add_to_ca() -> AppResult<RootCertStore> {
    let mut root_cert_store = rustls::RootCertStore::empty();
    let mut content = Vec::new();
    File::open("certs/shadow_ca.crt")
        .await?
        .read_to_end(&mut content)
        .await?;

    for cert in rustls_pemfile::certs(&mut Cursor::new(content)) {
        root_cert_store.add(cert?)?;
    }

    Ok(root_cert_store)
}
