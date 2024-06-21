pub mod client;
pub mod error;
mod misc;
pub mod server;

use error::ShadowError;
use remoc::codec;

pub use misc::get_version;
pub use misc::transfer;

pub type CallResult<T> = Result<T, ShadowError>;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum ObjectType {
    ClientClient(client::ClientClient<codec::Bincode>),
    ServerClient(server::ServerClient<codec::Bincode>),
}
