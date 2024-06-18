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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Display {
    /// The Display Name
    pub name: String,
    /// Unique identifier associated with the display.
    pub id: u32,
    /// The display x coordinate.
    pub x: i32,
    /// The display x coordinate.
    pub y: i32,
    /// The display pixel width.
    pub width: u32,
    /// The display pixel height.
    pub height: u32,
    /// Can be 0, 90, 180, 270, represents screen rotation in clock-wise degrees.
    pub rotation: f32,
    /// Output device's pixel scale factor.
    pub scale_factor: f32,
    /// The display refresh rate.
    pub frequency: f32,
    /// Whether the screen is the main screen
    pub is_primary: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Execute {
    pub output: String,
    pub status: String,
}
