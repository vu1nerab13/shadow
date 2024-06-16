use crate::misc;
use log::trace;
use remoc::{chmux::ChMuxError, codec, prelude::*};
use serde::{Deserialize, Serialize};
use shadow_common::{
    client::{self as sc, CallResult, Client},
    error::ShadowError,
    server as ss,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    sync::{RwLock, RwLockReadGuard},
    task::JoinHandle,
};

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
    pub client: Option<Arc<RwLock<sc::ClientClient<codec::Bincode>>>>,
    pub addr: SocketAddr,
    pub info: sc::SystemInfo,
    pub task: Option<JoinHandle<Result<(), ChMuxError<std::io::Error, std::io::Error>>>>,
}

impl ServerObj {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            client: None,
            cfg: ServerCfg::default(),
            info: sc::SystemInfo::default(),
            task: None,
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

    pub fn summary(&self) -> sc::SystemInfo {
        self.info.clone()
    }

    pub fn disconnect(&self) -> CallResult<()> {
        self.task
            .as_ref()
            .ok_or(ShadowError::DisconnectError)?
            .abort();

        Ok(())
    }

    pub async fn system_power(&self, action: sc::SystemPowerAction) -> CallResult<()> {
        self.get_client().await?.system_power(action).await
    }

    pub async fn get_installed_apps(&self) -> CallResult<Vec<sc::App>> {
        self.get_client().await?.get_installed_apps().await
    }

    pub async fn get_processes(&self) -> CallResult<Vec<sc::Process>> {
        self.get_client().await?.get_processes().await
    }

    pub async fn get_file_list<S: AsRef<str>>(&self, dir: S) -> CallResult<Vec<sc::File>> {
        self.get_client()
            .await?
            .get_file_list(dir.as_ref().into())
            .await
    }

    pub async fn get_file_content<S: AsRef<str>>(&self, file: S) -> CallResult<Vec<u8>> {
        self.get_client()
            .await?
            .get_file_content(file.as_ref().into())
            .await
    }

    pub async fn create_file<S: AsRef<str>>(&self, file: S) -> CallResult<()> {
        self.get_client()
            .await?
            .create_file(file.as_ref().into())
            .await
    }

    pub async fn open_file<S: AsRef<str>>(&self, file: S) -> CallResult<String> {
        self.get_client()
            .await?
            .open_file(file.as_ref().into())
            .await
    }

    pub async fn create_dir<S: AsRef<str>>(&self, dir: S) -> CallResult<()> {
        self.get_client()
            .await?
            .create_dir(dir.as_ref().into())
            .await
    }

    pub async fn write_file<S: AsRef<str>>(&self, file: S, content: Vec<u8>) -> CallResult<()> {
        self.get_client()
            .await?
            .write_file(file.as_ref().into(), content)
            .await
    }

    pub async fn delete_file<S: AsRef<str>>(&self, file: S) -> CallResult<()> {
        self.get_client()
            .await?
            .delete_file(file.as_ref().into())
            .await
    }

    pub async fn delete_dir_recursive<S: AsRef<str>>(&self, dir: S) -> CallResult<()> {
        self.get_client()
            .await?
            .delete_dir_recursive(dir.as_ref().into())
            .await
    }

    pub async fn kill_process(&self, pid: u32) -> CallResult<()> {
        self.get_client().await?.kill_process(pid).await
    }

    pub async fn get_display_info(&self) -> CallResult<Vec<sc::Display>> {
        self.get_client().await?.get_display_info().await
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
