mod socks5;

use super::Parameter;
use crate::network::ServerObj;
use anyhow::Result as AppResult;
use serde::{Deserialize, Serialize};
use shadow_common::error::ShadowError;
use std::{str::FromStr, sync::Arc};
use strum_macros::EnumString;
use tokio::sync::RwLock;
use warp::reply::Reply;

#[derive(EnumString, Deserialize, Serialize)]
pub enum ProxyOperation {
    #[strum(ascii_case_insensitive)]
    Socks5Open,
    #[strum(ascii_case_insensitive)]
    Socks5Close,
}

#[derive(Deserialize, Serialize)]
pub struct Proxy {
    op: String,
    #[serde(default = "default_addr")]
    addr: String,
    #[serde(default = "default_port")]
    port: u16,
}

fn default_addr() -> String {
    "0.0.0.0".into()
}

fn default_port() -> u16 {
    9999
}

impl Parameter for Proxy {
    type Operation = ProxyOperation;

    fn operation(&self) -> AppResult<Self::Operation> {
        Ok(Self::Operation::from_str(&self.op)?)
    }

    fn summarize() -> String {
        "proxy operation".into()
    }

    async fn dispatch(
        &self,
        op: Self::Operation,
        server_obj: Arc<RwLock<ServerObj>>,
    ) -> Result<Box<dyn Reply>, ShadowError> {
        let listen_addr = format!("{}:{}", self.addr, self.port).parse()?;

        match op {
            ProxyOperation::Socks5Open => socks5::open(server_obj, listen_addr).await,
            ProxyOperation::Socks5Close => socks5::close(server_obj, listen_addr).await,
        }
    }
}
