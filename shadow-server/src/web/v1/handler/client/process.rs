use super::Parameter;
use crate::network::ServerObj;
use anyhow::Result as AppResult;
use serde::{Deserialize, Serialize};
use shadow_common::error::ShadowError;
use std::{str::FromStr, sync::Arc};
use strum_macros::EnumString;
use tokio::sync::RwLock;
use warp::{
    http::StatusCode,
    reply::{self, Reply},
};

#[derive(EnumString, Deserialize, Serialize)]
pub enum ProcessOperation {
    #[strum(ascii_case_insensitive)]
    Query,
}

#[derive(Deserialize, Serialize)]
pub struct ProcessParameter {
    op: String,
}

impl Parameter for ProcessParameter {
    type Operation = ProcessOperation;

    fn operation(&self) -> AppResult<Self::Operation> {
        Ok(Self::Operation::from_str(&self.op)?)
    }

    fn summarize() -> String {
        "process operation".into()
    }

    async fn dispatch(
        &self,
        op: Self::Operation,
        server_obj: Arc<RwLock<ServerObj>>,
    ) -> Result<Box<dyn Reply>, ShadowError> {
        match op {
            ProcessOperation::Query => query_processes(server_obj).await,
        }
    }
}

async fn query_processes(
    server_obj: Arc<RwLock<ServerObj>>,
) -> Result<Box<dyn Reply>, ShadowError> {
    let processes = server_obj.read().await.get_processes().await?;

    Ok(Box::new(reply::with_status(
        reply::json(&processes),
        StatusCode::OK,
    )))
}
