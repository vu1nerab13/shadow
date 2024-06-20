mod socks5;

use super::Parameter;
use crate::network::ServerObj;
use anyhow::Result as AppResult;
use serde::{Deserialize, Serialize};
use shadow_common::error::ShadowError;
use std::{collections::HashMap, net::SocketAddr, str::FromStr, sync::Arc};
use strum_macros::EnumString;
use tokio::sync::RwLock;
use warp::{
    http::StatusCode,
    reply::{self, Reply},
};

#[derive(EnumString, Deserialize, Serialize)]
pub enum ProxyOperation {
    #[strum(ascii_case_insensitive)]
    Socks5,
}

#[derive(Deserialize, Serialize)]
pub struct ProxyParameter {
    op: String,
    addr: Vec<String>,
    port: u32,
}

impl Parameter for ProxyParameter {
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
        server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
    ) -> Result<Box<dyn Reply>, ShadowError> {
        match op {
            ProxyOperation::Socks5 => todo!(),
        }
    }
}
