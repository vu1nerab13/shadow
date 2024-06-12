mod file;
mod power;
mod query;

use super::super::error::{self, Error};
use crate::network::ServerObj;
use anyhow::Result as AppResult;
use file::FileParameter;
use power::PowerParameter;
use query::QueryParameter;
use shadow_common::error::ShadowError;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use warp::{
    filters::{any, body, query as fq, BoxedFilter},
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
        server_obj: Arc<RwLock<ServerObj>>,
    ) -> Result<Box<dyn Reply>, ShadowError>;

    fn summarize() -> String;

    async fn run(
        &self,
        addr: SocketAddr,
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
                    StatusCode::BAD_REQUEST,
                )))
            }
        };

        let lock = server_objs.read().await;
        let server_obj = match lock.get(&addr) {
            Some(o) => o.clone(),
            None => {
                return Ok(Box::new(reply::with_status(
                    reply::json(&Error {
                        message: addr.to_string(),
                        error: error::WebError::ClientNotFound,
                    }),
                    StatusCode::BAD_REQUEST,
                )))
            }
        };

        match self.dispatch(op, server_obj).await {
            Ok(r) => Ok(r),
            Err(e) => Ok(Box::new(reply::with_status(
                reply::json(&Error {
                    message: format!("error when performing {}", Self::summarize()),
                    error: error::WebError::ClientError(e),
                }),
                StatusCode::BAD_REQUEST,
            ))),
        }
    }
}

pub fn setup_routes(
    server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
) -> BoxedFilter<(impl Reply,)> {
    let prefix = warp::path!("v1" / "client" / SocketAddr / ..)
        .and(any::any().map(move || server_objs.clone()));

    let query = prefix
        .clone()
        .and(warp::get())
        .and(warp::path!("query"))
        .and(warp::path::end())
        .and(fq::query::<QueryParameter>())
        .and_then(run);

    let power = prefix
        .clone()
        .and(warp::post())
        .and(warp::path!("power"))
        .and(warp::path::end())
        .and(body::json::<PowerParameter>())
        .and_then(run);

    let file = prefix
        .clone()
        .and(warp::post())
        .and(warp::path!("file"))
        .and(warp::path::end())
        .and(body::json::<FileParameter>())
        .and_then(run);

    query.or(power).or(file).boxed()
}

async fn run<T>(
    addr: SocketAddr,
    server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
    param: T,
) -> Response<Box<dyn Reply>>
where
    T: Parameter,
{
    param.run(addr, server_objs).await
}
