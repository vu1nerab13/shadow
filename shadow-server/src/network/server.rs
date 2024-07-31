use crate::misc;
use log::{trace, warn};
use remoc::{
    codec::{self, Bincode},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use shadow_common::{
    client::{self as sc},
    error::ShadowError,
    server as ss, CallResult,
};
use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::Arc,
};
use tokio::sync::{oneshot, RwLock, RwLockReadGuard};

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerCfg {
    version: String,
}

impl Default for ServerCfg {
    fn default() -> Self {
        let version = misc::get_version();

        Self { version }
    }
}

#[derive(Debug)]
pub struct ServerObj {
    cfg: ServerCfg,
    pub client: Option<Arc<RwLock<sc::ClientClient<Bincode>>>>,
    pub addr: SocketAddr,
    pub info: sc::SystemInfo,
    pub signal_tx: Option<oneshot::Sender<bool>>,
    pub proxies: HashMap<SocketAddr, Option<oneshot::Sender<bool>>>,
}

impl ServerObj {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            client: None,
            cfg: ServerCfg::default(),
            info: sc::SystemInfo::default(),
            signal_tx: None,
            proxies: HashMap::new(),
        }
    }

    /// Get the client instance which connected to the server
    pub async fn get_client(
        &self,
    ) -> CallResult<RwLockReadGuard<sc::ClientClient<codec::Bincode>>> {
        Ok(self
            .client
            .as_ref()
            .ok_or(ShadowError::ClientNotFound)?
            .read()
            .await)
    }

    #[inline]
    pub fn ip(&self) -> IpAddr {
        self.addr.ip()
    }

    #[inline]
    pub fn summary(&self) -> sc::SystemInfo {
        self.info.clone()
    }

    pub fn shutdown_tasks(&mut self) {
        for (_, tx) in &mut self.proxies {
            if let Some(tx) = tx.take() {
                let _ = tx
                    .send(true)
                    .map_err(|_| warn!("{}", ShadowError::StopFailed.to_string()));
            }
        }
    }

    pub fn disconnect(&mut self) -> CallResult<()> {
        self.signal_tx
            .take()
            .ok_or(ShadowError::DisconnectError)?
            .send(true)
            .map_err(|_| ShadowError::StopFailed)?;

        Ok(())
    }
}

#[rtc::async_trait]
impl ss::Server for ServerObj {
    async fn handshake(&self) -> CallResult<ss::Handshake> {
        trace!("{}: handshake", self.addr);

        Ok(ss::Handshake {
            message: format!("{:#?}", self.cfg),
        })
    }
}
