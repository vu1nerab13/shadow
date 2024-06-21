use anyhow::Result as AppResult;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use shadow_common::error::ShadowError;
use std::io::Cursor;
use tokio::{fs::File, io::AsyncReadExt};

pub async fn load_certs(path: String) -> AppResult<Vec<CertificateDer<'static>>> {
    let mut content = Vec::new();
    File::open(path).await?.read_to_end(&mut content).await?;

    Ok(rustls_pemfile::certs(&mut Cursor::new(content)).collect::<Result<Vec<_>, _>>()?)
}

pub async fn load_keys(path: String) -> AppResult<PrivateKeyDer<'static>> {
    let mut content = Vec::new();
    File::open(path).await?.read_to_end(&mut content).await?;

    let key =
        rustls_pemfile::private_key(&mut Cursor::new(content))?.ok_or(ShadowError::NoPrivateKey)?;

    Ok(key.into())
}
