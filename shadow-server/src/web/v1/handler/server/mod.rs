use super::super::error::{self, Error};
use crate::network::ServerObj;
use anyhow::Result as AppResult;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, str::FromStr, sync::Arc};
use strum_macros::EnumString;
use tokio::sync::RwLock;
use warp::{
    filters::{any, query, BoxedFilter},
    http::StatusCode,
    path,
    reject::Rejection,
    reply::{self, Reply},
    Filter,
};

type Response<T> = Result<T, Rejection>;

#[derive(EnumString, Deserialize, Serialize)]
enum QueryOperation {
    #[strum(ascii_case_insensitive)]
    Clients,
}

#[derive(Deserialize, Serialize)]
struct QueryParameter {
    op: String,
}

impl Parameter for QueryParameter {
    type Operation = QueryOperation;

    fn operation(&self) -> AppResult<Self::Operation> {
        Ok(Self::Operation::from_str(&self.op)?)
    }

    async fn dispatch(
        &self,
        op: Self::Operation,
        server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
    ) -> Response<Box<dyn Reply>> {
        match op {
            QueryOperation::Clients => query_clients(server_objs).await,
        }
    }
}

trait Parameter {
    type Operation;

    fn operation(&self) -> AppResult<Self::Operation>;

    async fn dispatch(
        &self,
        op: Self::Operation,
        server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
    ) -> Response<Box<dyn Reply>>;

    async fn run(
        &self,
        server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
    ) -> Response<Box<dyn Reply>> {
        let op = match self.operation() {
            Ok(o) => o,
            Err(e) => {
                return Ok(Box::new(reply::with_status(
                    reply::json(&Error {
                        error: error::WebError::NoOp,
                        message: e.to_string(),
                    }),
                    StatusCode::OK,
                )))
            }
        };

        self.dispatch(op, server_objs).await
    }
}

pub fn setup_routes(
    server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
) -> BoxedFilter<(impl Reply,)> {
    let prefix = path!("v1" / "server" / ..).and(any::any().map(move || server_objs.clone()));

    let query = prefix
        .and(path!("query"))
        .and(path::end())
        .and(query::query::<QueryParameter>())
        .and_then(run);

    query.boxed()
}

async fn run<T>(
    server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
    param: T,
) -> Response<Box<dyn Reply>>
where
    T: Parameter,
{
    param.run(server_objs).await
}

async fn query_clients(
    server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
) -> Response<Box<dyn Reply>> {
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
