mod proxy;
mod query;

use crate::{network::ServerObj, web::error::Error};
use anyhow::Result as AppResult;
use query::QueryParameter;
use shadow_common::error::ShadowError;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use warp::{
    filters::{any, query as fq, BoxedFilter},
    http::StatusCode,
    reject::Rejection,
    reply::{self, Reply},
    Filter,
};

type Response<T> = Result<T, Rejection>;

trait Parameter {
    type Operation;

    fn operation(&self) -> AppResult<Self::Operation>;

    async fn dispatch(
        &self,
        op: Self::Operation,
        server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
    ) -> Result<Box<dyn Reply>, ShadowError>;

    fn summarize() -> String;

    async fn run(
        &self,
        server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
    ) -> Response<Box<dyn Reply>> {
        let op = match self.operation() {
            Ok(o) => o,
            Err(e) => {
                return Ok(Box::new(reply::with_status(
                    reply::json(&Error {
                        error: ShadowError::NoOp.to_string(),
                        message: e.to_string(),
                    }),
                    StatusCode::BAD_REQUEST,
                )))
            }
        };

        match self.dispatch(op, server_objs).await {
            Ok(r) => Ok(r),
            Err(e) => Ok(Box::new(reply::with_status(
                reply::json(&Error {
                    message: format!("error when performing {}", Self::summarize()),
                    error: e.to_string(),
                }),
                StatusCode::BAD_REQUEST,
            ))),
        }
    }
}

/// Setup server routes
pub fn setup_routes(
    server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
) -> BoxedFilter<(impl Reply,)> {
    let prefix = warp::path!("v1" / "server" / ..).and(any::any().map(move || server_objs.clone()));

    let query = prefix
        .clone()
        .and(warp::path!("query"))
        .and(warp::path::end())
        .and(fq::query::<QueryParameter>())
        .and_then(run);

    let proxy = prefix
        .clone()
        .and(warp::path!("proxy"))
        .and(warp::path::end())
        .and(fq::query::<QueryParameter>())
        .and_then(run);

    query.or(proxy).boxed()
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
