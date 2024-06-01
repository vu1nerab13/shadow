use crate::error::ShadowError;
use crabgrab::util::{Point, Rect, Size};
use remoc::prelude::*;
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
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "Point")]
pub struct PointDef {
    pub x: f64,
    pub y: f64,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "Size")]
pub struct SizeDef {
    pub width: f64,
    pub height: f64,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "Rect")]
pub struct RectDef {
    #[serde(with = "PointDef")]
    pub origin: Point,
    #[serde(with = "SizeDef")]
    pub size: Size,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Display {
    #[serde(with = "RectDef")]
    pub rect: Rect,
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

    async fn get_displays(&self) -> Result<Vec<Display>, ShadowError>;
}
