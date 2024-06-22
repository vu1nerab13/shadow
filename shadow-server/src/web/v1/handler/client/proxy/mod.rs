mod run;
mod socks5;

use super::Parameter;
use crate::network::ServerObj;
use anyhow::Result as AppResult;
use serde::{Deserialize, Serialize};
use shadow_common::CallResult;
use std::{str::FromStr, sync::Arc};
use strum_macros::{Display, EnumString};
use tokio::sync::RwLock;
use warp::reply::Reply;

#[derive(EnumString, Deserialize, Serialize)]
pub enum ProxyOperation {
    #[strum(ascii_case_insensitive)]
    Open,
    #[strum(ascii_case_insensitive)]
    Close,
}

#[derive(EnumString, Deserialize, Serialize, Display, Clone, Copy)]
pub enum ProxyType {
    #[strum(ascii_case_insensitive)]
    Socks5,
}

#[derive(Deserialize, Serialize)]
pub struct Proxy {
    op: String,
    r#type: String,
    #[serde(default = "default_addr")]
    addr: String,
    #[serde(default = "default_port")]
    port: u16,
    #[serde(default = "default_user")]
    user: String,
    #[serde(default = "default_password")]
    password: String,
}

fn default_addr() -> String {
    "0.0.0.0".into()
}

fn default_port() -> u16 {
    9999
}

fn default_user() -> String {
    "mitsuha".into()
}

fn default_password() -> String {
    "miyamizu".into()
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
    ) -> CallResult<Box<dyn Reply>> {
        let listen_addr = format!("{}:{}", self.addr, self.port).parse()?;

        match op {
            ProxyOperation::Open => {
                run::open(
                    server_obj,
                    listen_addr,
                    &self.r#type,
                    self.user.clone(),
                    self.password.clone(),
                )
                .await
            }
            ProxyOperation::Close => run::close(server_obj, listen_addr).await,
        }
    }
}
