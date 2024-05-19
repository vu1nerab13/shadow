use super::super::error;
use crate::network::ServerObj;
use anyhow::Result as AppResult;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, str::FromStr, sync::Arc};
use strum_macros::EnumString;
use tokio::sync::RwLock;
use warp::{http::StatusCode, reply, Rejection, Reply};

type Response<T> = Result<T, Rejection>;

#[derive(Deserialize, Serialize)]
pub struct ServerParam {
    op: String,
}

#[derive(EnumString, Deserialize, Serialize)]
enum ServerOperation {
    #[strum(ascii_case_insensitive)]
    Query,
}

pub async fn server_request(
    param: ServerParam,
    server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
) -> Response<Box<dyn Reply>> {
    #[derive(Serialize, Deserialize)]
    struct DispatchError {
        message: String,
        error: error::WebError,
    }

    let reply = match ServerOperation::from_str(&param.op) {
        Ok(op) => match server_op(op, server_objs).await {
            Ok(v) => v,
            Err(e) => Box::new(reply::with_status(
                reply::json(&DispatchError {
                    message: e.to_string(),
                    error: error::WebError::UnknownError,
                }),
                StatusCode::INTERNAL_SERVER_ERROR,
            )),
        },
        Err(e) => Box::new(reply::with_status(
            reply::json(&DispatchError {
                message: e.to_string(),
                error: error::WebError::NoOp,
            }),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    };

    Ok(reply)
}

/// Try to perform a operation on all clients
async fn server_op(
    op: ServerOperation,
    server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
) -> AppResult<Box<dyn Reply>> {
    match op {
        ServerOperation::Query => query_clients(server_objs).await,
    }
}

/// Query all clients that connected to the server
async fn query_clients(
    server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
) -> AppResult<Box<dyn Reply>> {
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
