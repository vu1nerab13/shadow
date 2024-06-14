use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

#[derive(Debug, Serialize, Deserialize)]
pub struct Handshake {
    pub message: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemInfo {
    pub system_name: String,
    pub kernel_version: String,
    pub os_version: String,
    pub host_name: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct App {
    pub name: String,
    pub publisher: String,
    pub version: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Process {
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub name: String,
    pub exe: String,
    pub start_time: u64,
    pub cwd: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct File {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
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
