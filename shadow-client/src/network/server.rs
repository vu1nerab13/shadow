use crate::misc;
use remoc::prelude::*;
use shadow_common::{client as sc, RtcResult};
use sysinfo::{Components, Disks, Networks, System};

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

#[derive(Default)]
pub struct ClientObj {
    cfg: ClientCfg,
}

#[rtc::async_trait]
impl sc::Client for ClientObj {
    async fn handshake(&self) -> RtcResult<sc::Handshake> {
        Ok(sc::Handshake {
            message: format!("{:#?}", self.cfg),
        })
    }

    async fn get_system_info(&self) -> RtcResult<sc::SystemInfo> {
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
}
