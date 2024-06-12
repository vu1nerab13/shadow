use crate::misc;
use log::trace;
use remoc::{chmux::ChMuxError, codec, prelude::*};
use serde::{Deserialize, Serialize};
use shadow_common::{
    client::{self as sc, Client, SystemPowerAction},
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
    ) -> Result<RwLockReadGuard<sc::ClientClient<codec::Bincode>>, ShadowError> {
        match &self.client {
            Some(c) => Ok(c.read().await),
            None => return Err(ShadowError::ClientNotFound),
        }
    }

    pub fn summary(&self) -> sc::SystemInfo {
        self.info.clone()
    }

    pub fn disconnect(&self) -> bool {
        let task = match &self.task {
            Some(t) => t,
            None => return false,
        };

        // Abort this task
        task.abort();

        true
    }

    pub async fn system_power(&self, action: SystemPowerAction) -> Result<bool, ShadowError> {
        self.get_client().await?.system_power(action).await
    }

    pub async fn get_installed_apps(&self) -> Result<Vec<sc::App>, ShadowError> {
        self.get_client().await?.get_installed_apps().await
    }

    pub async fn get_processes(&self) -> Result<Vec<sc::Process>, ShadowError> {
        self.get_client().await?.get_processes().await
    }

    pub async fn get_displays(&self) -> Result<Vec<sc::Display>, ShadowError> {
        self.get_client().await?.get_displays().await
    }

    pub async fn get_file_list<S: AsRef<str>>(&self, dir: S) -> Result<Vec<sc::File>, ShadowError> {
        self.get_client()
            .await?
            .get_file_list(dir.as_ref().into())
            .await
    }

    pub async fn get_file_content<S: AsRef<str>>(&self, file: S) -> Result<Vec<u8>, ShadowError> {
        self.get_client()
            .await?
            .get_file_content(file.as_ref().into())
            .await
    }

    pub async fn create_file<S: AsRef<str>>(&self, file: S) -> Result<(), ShadowError> {
        self.get_client()
            .await?
            .create_file(file.as_ref().into())
            .await
    }

    pub async fn write_file<S: AsRef<str>>(
        &self,
        file: S,
        content: Vec<u8>,
    ) -> Result<(), ShadowError> {
        self.get_client()
            .await?
            .write_file(file.as_ref().into(), content)
            .await
    }
}

#[rtc::async_trait]
impl ss::Server for ServerObj {
    async fn handshake(&self) -> Result<ss::Handshake, ShadowError> {
        trace!("{}: handshake", self.addr);

        Ok(ss::Handshake {
            message: format!("{:#?}", self.cfg),
        })
    }
}
