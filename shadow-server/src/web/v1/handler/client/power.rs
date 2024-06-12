use super::{
    super::super::error::{self, Error},
    Parameter,
};
use crate::network::ServerObj;
use anyhow::Result as AppResult;
use serde::{Deserialize, Serialize};
use shadow_common::{client::SystemPowerAction, error::ShadowError};
use std::{str::FromStr, sync::Arc};
use tokio::sync::RwLock;
use warp::{
    http::StatusCode,
    reply::{self, Reply},
};

#[derive(Deserialize, Serialize)]
pub struct PowerParameter {
    op: String,
}

impl Parameter for PowerParameter {
    type Operation = SystemPowerAction;

    fn operation(&self) -> AppResult<Self::Operation> {
        Ok(Self::Operation::from_str(&self.op)?)
    }

    fn summarize() -> String {
        "power operation".into()
    }

    async fn dispatch(
        &self,
        op: Self::Operation,
        server_obj: Arc<RwLock<ServerObj>>,
    ) -> Result<Box<dyn Reply>, ShadowError> {
        let (message, error, code) = match server_obj.read().await.system_power(op).await {
            Ok(b) => match b {
                true => ("".into(), error::WebError::Success, StatusCode::OK),
                false => (
                    "can not perform power action".into(),
                    error::WebError::UnknownError,
                    StatusCode::BAD_REQUEST,
                ),
            },
            Err(e) => (
                e.to_string(),
                error::WebError::UnknownError,
                StatusCode::BAD_REQUEST,
            ),
        };

        return Ok(Box::new(reply::with_status(
            reply::json(&Error { message, error }),
            code,
        )));
    }
}
