use crate::{error::ShadowError, RtcResult};
use remoc::prelude::*;
use uuid::Uuid;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Handshake {
    pub message: String,
}

#[rtc::remote]
pub trait Server {
    async fn handshake(&self, uuid: Uuid) -> Result<Handshake, ShadowError>;
}
