use crate::misc;
use log::{trace, warn};
use rch::oneshot::Receiver;
use remoc::{
    codec::{self, Bincode},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use shadow_common::{
    client::{self as sc, Client},
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
    pub fn get_ip(&self) -> IpAddr {
        self.addr.ip()
    }

    #[inline]
    pub fn summary(&self) -> sc::SystemInfo {
        self.info.clone()
    }

    #[inline]
    pub fn shutdown_tasks(&mut self) {
        for (_, tx) in &mut self.proxies {
            if let Some(tx) = tx.take() {
                let _ = tx
                    .send(true)
                    .map_err(|_| warn!("{}", ShadowError::StopFailed.to_string()));
            }
        }
    }

    #[inline]
    pub fn disconnect(&mut self) -> CallResult<()> {
        self.signal_tx
            .take()
            .ok_or(ShadowError::DisconnectError)?
            .send(true)
            .map_err(|_| ShadowError::StopFailed)?;

        Ok(())
    }

    #[inline]
    pub async fn system_power(&self, action: sc::SystemPowerAction) -> CallResult<()> {
        self.get_client().await?.system_power(action).await
    }

    #[inline]
    pub async fn get_installed_apps(&self) -> CallResult<Vec<sc::App>> {
        self.get_client().await?.get_installed_apps().await
    }

    #[inline]
    pub async fn get_processes(&self) -> CallResult<Vec<sc::Process>> {
        self.get_client().await?.get_processes().await
    }

    #[inline]
    pub async fn get_file_list<S: AsRef<str>>(&self, dir: S) -> CallResult<Vec<sc::File>> {
        self.get_client()
            .await?
            .get_file_list(dir.as_ref().into())
            .await
    }

    #[inline]
    pub async fn get_file_content<S: AsRef<str>>(&self, file: S) -> CallResult<Vec<u8>> {
        self.get_client()
            .await?
            .get_file_content(file.as_ref().into())
            .await
    }

    #[inline]
    pub async fn create_file<S: AsRef<str>>(&self, file: S) -> CallResult<()> {
        self.get_client()
            .await?
            .create_file(file.as_ref().into())
            .await
    }

    #[inline]
    pub async fn open_file<S: AsRef<str>>(&self, file: S) -> CallResult<sc::Execute> {
        self.get_client()
            .await?
            .open_file(file.as_ref().into())
            .await
    }

    #[inline]
    pub async fn create_dir<S: AsRef<str>>(&self, dir: S) -> CallResult<()> {
        self.get_client()
            .await?
            .create_dir(dir.as_ref().into())
            .await
    }

    #[inline]
    pub async fn write_file<S: AsRef<str>>(&self, file: S, content: Vec<u8>) -> CallResult<()> {
        self.get_client()
            .await?
            .write_file(file.as_ref().into(), content)
            .await
    }

    #[inline]
    pub async fn delete_file<S: AsRef<str>>(&self, file: S) -> CallResult<()> {
        self.get_client()
            .await?
            .delete_file(file.as_ref().into())
            .await
    }

    #[inline]
    pub async fn delete_dir_recursive<S: AsRef<str>>(&self, dir: S) -> CallResult<()> {
        self.get_client()
            .await?
            .delete_dir_recursive(dir.as_ref().into())
            .await
    }

    #[inline]
    pub async fn kill_process(&self, pid: u32) -> CallResult<()> {
        self.get_client().await?.kill_process(pid).await
    }

    #[inline]
    pub async fn get_display_info(&self) -> CallResult<Vec<sc::Display>> {
        self.get_client().await?.get_display_info().await
    }

    #[inline]
    pub async fn proxy(
        &self,
        target_addr: SocketAddr,
        sender: rch::bin::Sender,
        receiver: rch::bin::Receiver,
    ) -> CallResult<Receiver<bool, Bincode>> {
        self.get_client()
            .await?
            .proxy(target_addr, sender, receiver)
            .await
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
