use super::{super::super::error::Error, Parameter};
use crate::network::ServerObj;
use anyhow::Result as AppResult;
use serde::{Deserialize, Serialize};
use shadow_common::{client::SystemPowerAction, error::ShadowError, CallResult};
use std::{str::FromStr, sync::Arc};
use tokio::sync::RwLock;
use warp::{
    http::StatusCode,
    reply::{self, Reply},
};

#[derive(Deserialize, Serialize)]
pub struct Power {
    op: String,
}

impl Parameter for Power {
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
    ) -> CallResult<Box<dyn Reply>> {
        let (message, error, code) = match server_obj.read().await.system_power(op).await {
            Ok(_) => (
                "power action successfully completed".into(),
                ShadowError::Success,
                StatusCode::OK,
            ),
            Err(e) => (e.to_string(), e, StatusCode::BAD_REQUEST),
        };
        let error = error.to_string();

        return Ok(Box::new(reply::with_status(
            reply::json(&Error { message, error }),
            code,
        )));
    }
}
