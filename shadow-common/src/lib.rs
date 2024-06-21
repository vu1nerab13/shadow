pub mod client;
pub mod error;
mod misc;
pub mod server;

pub use misc::get_version;
pub use misc::transfer;

use remoc::{codec, rtc};

pub type RtcResult<T> = Result<T, rtc::CallError>;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum ObjectType {
    ClientClient(client::ClientClient<codec::Bincode>),
    ServerClient(server::ServerClient<codec::Bincode>),
}
