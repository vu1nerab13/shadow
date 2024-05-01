use crate::misc;
use log::trace;
use remoc::{codec, prelude::*};
use shadow_common::{
    client::{ClientClient, SystemInfo},
    error::ShadowError,
    server as ss,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;

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
    pub client: Option<Arc<RwLock<ClientClient<codec::Bincode>>>>,
    pub addr: SocketAddr,
    pub info: SystemInfo,
}

impl ServerObj {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            client: Option::None,
            cfg: ServerCfg::default(),
            info: SystemInfo::default(),
        }
    }

    pub fn summary(&self) -> String {
        format!("{:?}", self)
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
