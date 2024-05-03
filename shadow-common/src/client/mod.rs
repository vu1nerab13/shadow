use crate::error::ShadowError;
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

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct App {
    pub name: String,
    pub publisher: String,
    pub version: String,
}

#[rtc::remote]
pub trait Client {
    async fn handshake(&self) -> Result<Handshake, ShadowError>;

    async fn get_system_info(&self) -> Result<SystemInfo, ShadowError>;

    async fn system_shutdown(&self) -> Result<bool, ShadowError>;

    async fn system_logout(&self) -> Result<bool, ShadowError>;

    async fn system_reboot(&self) -> Result<bool, ShadowError>;

    async fn system_hibernate(&self) -> Result<bool, ShadowError>;

    async fn system_sleep(&self) -> Result<bool, ShadowError>;

    async fn get_installed_apps(&self) -> Result<Vec<App>, ShadowError>;
}
