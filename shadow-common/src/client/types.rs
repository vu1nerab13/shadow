use super::shim;
use crabgrab::{
    capture_stream::CapturePixelFormat,
    util::{Point, Rect, Size},
};
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

#[derive(Debug, Serialize, Deserialize)]
pub struct PixelFormat(#[serde(with = "shim")] pub CapturePixelFormat);

#[derive(Serialize, Deserialize)]
pub enum FrameType {
    BgraUnorm8x4(Vec<[u8; 4]>),
    RgbaUnormPacked1010102(Vec<u32>),
    RgbaF16x4,
    YCbCr,
}

#[derive(Serialize, Deserialize)]
pub struct Frame {
    pub frame_type: FrameType,
    pub height: usize,
    pub width: usize,
}

impl From<CapturePixelFormat> for PixelFormat {
    fn from(value: CapturePixelFormat) -> Self {
        Self { 0: value }
    }
}

impl Into<CapturePixelFormat> for PixelFormat {
    fn into(self) -> CapturePixelFormat {
        self.0
    }
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
