mod shim;
mod types;

use crate::CallResult;
use rch::oneshot::Receiver;
use remoc::{codec::Bincode, prelude::*};
use std::net::SocketAddr;

pub use types::*;

#[rtc::remote]
pub trait Client {
    async fn handshake(&self) -> CallResult<Handshake>;

    async fn get_system_info(&self) -> CallResult<SystemInfo>;

    async fn system_power(&self, action: SystemPowerAction) -> CallResult<()>;

    async fn get_installed_apps(&self) -> CallResult<Vec<App>>;

    async fn get_processes(&self) -> CallResult<Vec<Process>>;

    async fn kill_process(&self, pid: u32) -> CallResult<()>;

    async fn get_file_list(&self, dir: String) -> CallResult<Vec<File>>;

    async fn get_file_content(&self, file_path: String) -> CallResult<Vec<u8>>;

    async fn create_file(&self, file_path: String) -> CallResult<()>;

    async fn open_file(&self, file_path: String) -> CallResult<Execute>;

    async fn create_dir(&self, dir_path: String) -> CallResult<()>;

    async fn delete_file(&self, file_path: String) -> CallResult<()>;

    async fn delete_dir_recursive(&self, dir_path: String) -> CallResult<()>;

    async fn write_file(&self, file_path: String, content: Vec<u8>) -> CallResult<()>;

    async fn get_display_info(&self) -> CallResult<Vec<Display>>;

    async fn proxy(
        &self,
        target_addr: SocketAddr,
        sender: rch::bin::Sender,
        receiver: rch::bin::Receiver,
    ) -> CallResult<Receiver<bool, Bincode>>;
}
