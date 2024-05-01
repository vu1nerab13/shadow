pub mod client;
pub mod error;
mod misc;
pub mod server;

pub use misc::get_version;

use remoc::{codec, rtc};
use uuid::Uuid;

pub type RtcResult<T> = Result<T, rtc::CallError>;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum ObjectType {
    ClientClient(client::ClientClient<codec::Bincode>),
    ServerClient(server::ServerClient<codec::Bincode>),
    Uuid(Uuid),
}
