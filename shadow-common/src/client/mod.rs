use crate::error::ShadowError;
use remoc::prelude::*;
use strum_macros::EnumString;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Handshake {
    pub message: String,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SystemInfo {
    pub system_name: String,
    pub kernel_version: String,
    pub os_version: String,
    pub host_name: String,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct App {
    pub name: String,
    pub publisher: String,
    pub version: String,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Process {
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub name: String,
    pub exe: String,
    pub start_time: u64,
    pub cwd: String,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct File {
    pub name: String,
    pub is_dir: bool,
}

#[derive(EnumString, Debug, serde::Serialize, serde::Deserialize)]
pub enum SystemPowerAction {
    #[strum(ascii_case_insensitive)]
    Shutdown,
    #[strum(ascii_case_insensitive)]
    Reboot,
    #[strum(ascii_case_insensitive)]
    Logout,
    #[strum(ascii_case_insensitive)]
    Sleep,
    #[strum(ascii_case_insensitive)]
    Hibernate,
}

#[rtc::remote]
pub trait Client {
    async fn handshake(&self) -> Result<Handshake, ShadowError>;

    async fn get_system_info(&self) -> Result<SystemInfo, ShadowError>;

    async fn system_power(&self, action: SystemPowerAction) -> Result<bool, ShadowError>;

    async fn get_installed_apps(&self) -> Result<Vec<App>, ShadowError>;

    async fn get_processes(&self) -> Result<Vec<Process>, ShadowError>;

    async fn get_file_list(&self, dir: String) -> Result<Vec<File>, ShadowError>;
}
