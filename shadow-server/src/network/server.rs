use crate::misc;
use log::trace;
use remoc::{chmux::ChMuxError, codec, prelude::*};
use shadow_common::{
    client::{self as sc, Client},
    error::ShadowError,
    server as ss,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::{sync::RwLock, task::JoinHandle};

#[allow(dead_code)]
#[derive(Debug)]
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

    pub fn summary(&self) -> String {
        format!("{:?}", self)
    }

    pub fn disconnect(&self) -> bool {
        let task = match &self.task {
            Some(t) => t,
            None => return false,
        };

        task.abort();

        true
    }

    pub async fn system_shutdown(&self) -> bool {
        let client = match &self.client {
            Some(c) => c,
            None => return false,
        };

        match client.read().await.system_shutdown().await {
            Ok(_) => true,
            Err(_) => false,
        }
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
