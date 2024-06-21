use crate::CallResult;
use remoc::prelude::*;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Handshake {
    pub message: String,
}

#[rtc::remote]
pub trait Server {
    async fn handshake(&self) -> CallResult<Handshake>;
}
