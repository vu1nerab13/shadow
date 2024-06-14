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

    async fn kill_process(&self, pid: u32) -> Result<(), ShadowError>;

    async fn get_file_list(&self, dir: String) -> Result<Vec<File>, ShadowError>;

    async fn get_file_content(&self, file_path: String) -> Result<Vec<u8>, ShadowError>;

    async fn create_file(&self, file_path: String) -> Result<(), ShadowError>;

    async fn open_file(&self, file_path: String) -> Result<String, ShadowError>;

    async fn create_dir(&self, dir_path: String) -> Result<(), ShadowError>;

    async fn delete_file(&self, file_path: String) -> Result<(), ShadowError>;

    async fn delete_dir_recursive(&self, dir_path: String) -> Result<(), ShadowError>;

    async fn write_file(&self, file_path: String, content: Vec<u8>) -> Result<(), ShadowError>;
}
