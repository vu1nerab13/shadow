use super::Parameter;
use crate::network::ServerObj;
use anyhow::Result as AppResult;
use serde::{Deserialize, Serialize};
use shadow_common::CallResult;
use std::{str::FromStr, sync::Arc};
use strum_macros::EnumString;
use tokio::sync::RwLock;
use warp::{
    http::StatusCode,
    reply::{self, Reply},
};

#[derive(EnumString, Deserialize, Serialize)]
pub enum AppOperation {
    #[strum(ascii_case_insensitive)]
    Enumerate,
}

#[derive(Deserialize, Serialize)]
pub struct App {
    op: String,
}

impl Parameter for App {
    type Operation = AppOperation;

    fn operation(&self) -> AppResult<Self::Operation> {
        Ok(Self::Operation::from_str(&self.op)?)
    }

    fn summarize() -> String {
        "app operation".into()
    }

    async fn dispatch(
        &self,
        op: Self::Operation,
        server_obj: Arc<RwLock<ServerObj>>,
    ) -> CallResult<Box<dyn Reply>> {
        match op {
            AppOperation::Enumerate => query_apps(server_obj).await,
        }
    }
}

async fn query_apps(server_obj: Arc<RwLock<ServerObj>>) -> CallResult<Box<dyn Reply>> {
    let apps = server_obj.read().await.get_installed_apps().await?;

    Ok(Box::new(reply::with_status(
        reply::json(&apps),
        StatusCode::OK,
    )))
}
