use crate::RtcResult;
use remoc::prelude::*;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Handshake {
    pub message: String,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct SystemInfo {
    pub system_name: String,
    pub kernel_version: String,
    pub os_version: String,
    pub host_name: String,
    pub components: String,
    pub disks: String,
    pub networks: String,
    pub system: String,
}

#[rtc::remote]
pub trait Client {
    async fn handshake(&self) -> RtcResult<Handshake>;

    async fn get_system_info(&self) -> RtcResult<SystemInfo>;
}
