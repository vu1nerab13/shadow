pub mod client;
mod misc;
pub mod server;

pub use misc::get_version;

use remoc::codec;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum ObjectType {
    ClientClient(client::ClientClient<codec::Bincode>),
    ServerClient(server::ServerClient<codec::Bincode>),
}
