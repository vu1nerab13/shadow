use anyhow::Result as AppResult;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use shadow_common::error::ShadowError;
use std::{io::Cursor, path::Path};
use tokio::{fs::File, io::AsyncReadExt};

pub async fn load_certs(path: &Path) -> AppResult<Vec<CertificateDer>> {
    let mut content = Vec::new();
    File::open(path).await?.read_to_end(&mut content).await?;

    Ok(rustls_pemfile::certs(&mut Cursor::new(content)).collect::<Result<Vec<_>, _>>()?)
}

pub async fn load_keys(path: &Path) -> AppResult<PrivateKeyDer> {
    let mut content = Vec::new();
    File::open(path).await?.read_to_end(&mut content).await?;

    let key = match rustls_pemfile::private_key(&mut Cursor::new(content))? {
        Some(k) => k,
        None => return Err(ShadowError::NoPrivateKey.into()),
    };

    Ok(key.into())
}
