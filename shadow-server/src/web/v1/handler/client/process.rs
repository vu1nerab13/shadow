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
    Enumerate,
    #[strum(ascii_case_insensitive)]
    Kill,
}

#[derive(Deserialize, Serialize)]
pub struct Process {
    op: String,
    pid: Option<u32>,
}

impl Parameter for Process {
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
            ProcessOperation::Enumerate => query_processes(server_obj).await,
            ProcessOperation::Kill => kill_process(server_obj, &self.pid).await,
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

async fn kill_process(
    server_obj: Arc<RwLock<ServerObj>>,
    pid: &Option<u32>,
) -> Result<Box<dyn Reply>, ShadowError> {
    let pid = super::require(pid.clone(), "process id")?;

    server_obj.read().await.kill_process(pid).await?;

    super::success()
}
