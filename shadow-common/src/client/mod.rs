mod shim;
mod types;

use crate::error::ShadowError;
use remoc::prelude::*;
pub use types::*;

#[rtc::remote]
pub trait Client {
    async fn handshake(&self) -> Result<Handshake, ShadowError>;

    async fn get_system_info(&self) -> Result<SystemInfo, ShadowError>;

    async fn system_power(&self, action: SystemPowerAction) -> Result<bool, ShadowError>;

    async fn get_installed_apps(&self) -> Result<Vec<App>, ShadowError>;

    async fn get_processes(&self) -> Result<Vec<Process>, ShadowError>;

    async fn get_file_list(&self, dir: String) -> Result<Vec<File>, ShadowError>;

    async fn get_file_content(&self, file_path: String) -> Result<Vec<u8>, ShadowError>;

    async fn create_file(&self, file_path: String) -> Result<(), ShadowError>;

    async fn get_displays(&self) -> Result<Vec<Display>, ShadowError>;

    async fn get_pixel_formats(&self) -> Result<Vec<PixelFormat>, ShadowError>;

    async fn get_screenshot(
        &self,
        n_display: usize,
        format: PixelFormat,
    ) -> Result<Frame, ShadowError>;
}
