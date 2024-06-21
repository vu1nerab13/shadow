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
pub enum DisplayOperation {
    #[strum(ascii_case_insensitive)]
    Enumerate,
}

#[derive(Deserialize, Serialize)]
pub struct Display {
    op: String,
}

impl Parameter for Display {
    type Operation = DisplayOperation;

    fn operation(&self) -> AppResult<Self::Operation> {
        Ok(Self::Operation::from_str(&self.op)?)
    }

    fn summarize() -> String {
        "display operation".into()
    }

    async fn dispatch(
        &self,
        op: Self::Operation,
        server_obj: Arc<RwLock<ServerObj>>,
    ) -> CallResult<Box<dyn Reply>> {
        match op {
            DisplayOperation::Enumerate => query_displays(server_obj).await,
        }
    }
}

async fn query_displays(server_obj: Arc<RwLock<ServerObj>>) -> CallResult<Box<dyn Reply>> {
    let displays = server_obj.read().await.get_display_info().await?;

    Ok(Box::new(reply::with_status(
        reply::json(&displays),
        StatusCode::OK,
    )))
}
