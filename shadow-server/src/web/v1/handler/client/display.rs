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
pub enum DisplayOperation {}

#[derive(Deserialize, Serialize)]
pub struct DisplayParameter {
    op: String,
}

impl Parameter for DisplayParameter {
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
        _server_obj: Arc<RwLock<ServerObj>>,
    ) -> Result<Box<dyn Reply>, ShadowError> {
        match op {}
    }
}
