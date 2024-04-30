use remoc::prelude::*;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Handshake {
    pub message: String,
}

#[rtc::remote]
pub trait Client {
    async fn handshake(&self) -> Result<Handshake, rtc::CallError>;
}
