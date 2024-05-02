use crate::misc;
use remoc::{codec, prelude::*};
use shadow_common::{client as sc, error::ShadowError, server as ss};
use std::sync::Arc;
use sysinfo::{Components, Disks, Networks, System};
use tokio::sync::RwLock;

#[allow(dead_code)]
#[derive(Debug)]
pub struct ClientCfg {
    version: String,
}

impl Default for ClientCfg {
    fn default() -> Self {
        let version = misc::get_version();

        Self { version }
    }
}

pub struct ClientObj {
    cfg: ClientCfg,
    pub client: Option<Arc<RwLock<ss::ServerClient<codec::Bincode>>>>,
}

impl ClientObj {
    pub fn new() -> Self {
        Self {
            cfg: ClientCfg::default(),
            client: None,
        }
    }
}

#[rtc::async_trait]
impl sc::Client for ClientObj {
    async fn handshake(&self) -> Result<sc::Handshake, ShadowError> {
        Ok(sc::Handshake {
            message: format!("{:#?}", self.cfg),
        })
    }

    async fn get_system_info(&self) -> Result<sc::SystemInfo, ShadowError> {
        let mut system = System::new_all();
        system.refresh_all();

        let system_name = System::name().unwrap_or_default();
        let kernel_version = System::kernel_version().unwrap_or_default();
        let os_version = System::os_version().unwrap_or_default();
        let host_name = System::host_name().unwrap_or_default();

        let system = format!("{:?}", system);
        let disks = format!("{:?}", Disks::new_with_refreshed_list());
        let networks = format!("{:?}", Networks::new_with_refreshed_list());
        let components = format!("{:?}", Components::new_with_refreshed_list());

        Ok(sc::SystemInfo {
            system_name,
            kernel_version,
            os_version,
            host_name,
            components,
            disks,
            networks,
            system,
        })
    }

    async fn shutdown(&self) -> Result<(), ShadowError> {
        unimplemented!();
    }
}
