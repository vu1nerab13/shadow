use super::super::error::{self, Error};
use crate::network::ServerObj;
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

pub fn setup_routes(
    server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
) -> BoxedFilter<(impl Reply,)> {
    let prefix = path!("v1" / "server" / ..).and(any::any().map(move || server_objs.clone()));

    let query = prefix
        .and(path!("query"))
        .and(path::end())
        .and(query::query())
        .and_then(query);

    query.boxed()
}

async fn query(
    server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
    param: QueryParameter,
) -> Response<Box<dyn Reply>> {
    let op = match QueryOperation::from_str(&param.op) {
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

    match op {
        QueryOperation::Clients => query_clients(server_objs).await,
    }
}

async fn query_clients(
    server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
) -> Response<Box<dyn Reply>> {
    #[derive(Serialize, Deserialize)]
    struct Query {
        count: usize,
        peers: Vec<String>,
    }

    let server_objs = server_objs.read().await;

    Ok(Box::new(reply::with_status(
        reply::json(&Query {
            count: server_objs.len(),
            peers: server_objs
                .keys()
                .map(|addr: &SocketAddr| addr.to_string())
                .collect(),
        }),
        StatusCode::OK,
    )))
}
