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
pub enum QueryOperation {
    #[strum(ascii_case_insensitive)]
    Summary,
    #[strum(ascii_case_insensitive)]
    Apps,
    #[strum(ascii_case_insensitive)]
    Processes,
    #[strum(ascii_case_insensitive)]
    Displays,
}

#[derive(Deserialize, Serialize)]
pub struct QueryParameter {
    op: String,
}

impl Parameter for QueryParameter {
    type Operation = QueryOperation;

    fn operation(&self) -> AppResult<Self::Operation> {
        Ok(Self::Operation::from_str(&self.op)?)
    }

    fn summarize() -> String {
        "query operation".into()
    }

    async fn dispatch(
        &self,
        op: Self::Operation,
        server_obj: Arc<RwLock<ServerObj>>,
    ) -> Result<Box<dyn Reply>, ShadowError> {
        match op {
            QueryOperation::Summary => summarize_client(server_obj).await,
            QueryOperation::Apps => get_client_apps(server_obj).await,
            QueryOperation::Processes => get_client_processes(server_obj).await,
            QueryOperation::Displays => get_client_displays(server_obj).await,
        }
    }
}

async fn summarize_client(
    server_obj: Arc<RwLock<ServerObj>>,
) -> Result<Box<dyn Reply>, ShadowError> {
    Ok(Box::new(reply::with_status(
        reply::json(&server_obj.read().await.summary()),
        StatusCode::OK,
    )))
}

async fn get_client_apps(
    server_obj: Arc<RwLock<ServerObj>>,
) -> Result<Box<dyn Reply>, ShadowError> {
    let apps = server_obj.read().await.get_installed_apps().await?;

    Ok(Box::new(reply::with_status(
        reply::json(&apps),
        StatusCode::OK,
    )))
}

async fn get_client_processes(
    server_obj: Arc<RwLock<ServerObj>>,
) -> Result<Box<dyn Reply>, ShadowError> {
    let processes = server_obj.read().await.get_processes().await?;

    Ok(Box::new(reply::with_status(
        reply::json(&processes),
        StatusCode::OK,
    )))
}

async fn get_client_displays(
    server_obj: Arc<RwLock<ServerObj>>,
) -> Result<Box<dyn Reply>, ShadowError> {
    let displays = server_obj.read().await.get_displays().await?;

    Ok(Box::new(reply::with_status(
        reply::json(&displays),
        StatusCode::OK,
    )))
}
