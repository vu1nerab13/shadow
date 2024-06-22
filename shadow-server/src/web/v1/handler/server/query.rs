use super::Parameter;
use crate::network::ServerObj;
use anyhow::Result as AppResult;
use serde::{Deserialize, Serialize};
use shadow_common::CallResult;
use std::{collections::HashMap, net::SocketAddr, str::FromStr, sync::Arc};
use strum_macros::EnumString;
use tokio::sync::RwLock;
use warp::{
    http::StatusCode,
    reply::{self, Reply},
};

#[derive(EnumString, Deserialize, Serialize)]
pub enum QueryOperation {
    #[strum(ascii_case_insensitive)]
    Clients,
}

#[derive(Deserialize, Serialize)]
pub struct Query {
    op: String,
}

impl Parameter for Query {
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
        server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
    ) -> CallResult<Box<dyn Reply>> {
        match op {
            QueryOperation::Clients => query_clients(server_objs).await,
        }
    }
}

async fn query_clients(
    server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
) -> CallResult<Box<dyn Reply>> {
    let server_objs = server_objs.read().await;

    Ok(Box::new(reply::with_status(
        reply::json(
            &server_objs
                .keys()
                .map(|addr: &SocketAddr| addr.to_string())
                .collect::<Vec<_>>(),
        ),
        StatusCode::OK,
    )))
}
