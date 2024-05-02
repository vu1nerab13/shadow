use crate::{network::ServerObj, web::error};
use anyhow::Result as AppResult;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, str::FromStr, sync::Arc};
use strum_macros::EnumString;
use tokio::sync::RwLock;
use warp::{http::StatusCode, reply, Rejection, Reply};

type Response<T> = Result<T, Rejection>;

#[derive(EnumString)]
enum Operation {
    #[strum(ascii_case_insensitive)]
    Summary,
    #[strum(ascii_case_insensitive)]
    Shutdown,
}

pub async fn client_operation(
    addr: String,
    op: Option<String>,
    server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
) -> Response<impl Reply> {
    #[derive(Serialize, Deserialize)]
    struct UnknownError {
        message: String,
        error: error::WebError,
    }

    let (reply, status) = match match addr.as_str() {
        "query" if op == None => query_clients(server_objs).await,
        _ => try_client_op(addr, op, server_objs).await,
    } {
        Ok(v) => v,
        Err(e) => (
            serde_json::to_string(&UnknownError {
                message: e.to_string(),
                error: error::WebError::UnknownError,
            })
            .unwrap_or_default(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    };

    Ok(reply::with_status(reply, status))
}

async fn query_clients(
    server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
) -> AppResult<(String, StatusCode)> {
    #[derive(Serialize, Deserialize)]
    struct Query {
        count: usize,
        peers: Vec<String>,
    }

    let server_objs = server_objs.read().await;

    Ok((
        serde_json::to_string(&Query {
            count: server_objs.len(),
            peers: server_objs
                .keys()
                .map(|addr: &SocketAddr| addr.to_string())
                .collect(),
        })?,
        StatusCode::OK,
    ))
}

async fn try_client_op(
    addr: String,
    op: Option<String>,
    server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
) -> AppResult<(String, StatusCode)> {
    #[derive(Serialize, Deserialize)]
    struct QueryClient {
        error: error::WebError,
    }

    let op = match op {
        Some(o) => Operation::from_str(&o)?,
        None => {
            return Ok((
                serde_json::to_string(&QueryClient {
                    error: error::WebError::NoOp,
                })?,
                StatusCode::BAD_REQUEST,
            ))
        }
    };
    let lock = server_objs.read().await;
    let server_obj = match lock.get(&addr.parse()?) {
        Some(c) => c,
        None => {
            return Ok((
                serde_json::to_string(&QueryClient {
                    error: error::WebError::ClientNotFound,
                })?,
                StatusCode::NOT_FOUND,
            ))
        }
    };

    match op {
        Operation::Summary => get_client_summary(server_obj).await,
        Operation::Shutdown => get_client_shutdown(server_obj).await,
    }
}

async fn get_client_summary(
    server_obj: &Arc<RwLock<ServerObj>>,
) -> AppResult<(String, StatusCode)> {
    #[derive(Serialize, Deserialize)]
    struct GetSummary {
        summary: String,
        error: error::WebError,
    }

    Ok((
        serde_json::to_string(&GetSummary {
            summary: server_obj.read().await.summary(),
            error: error::WebError::Success,
        })?,
        StatusCode::OK,
    ))
}

async fn get_client_shutdown(
    server_obj: &Arc<RwLock<ServerObj>>,
) -> AppResult<(String, StatusCode)> {
    #[derive(Serialize, Deserialize)]
    struct Shutdown {
        error: error::WebError,
    }

    let error = match server_obj.read().await.shutdown() {
        true => error::WebError::Success,
        false => error::WebError::UnknownError,
    };

    Ok((serde_json::to_string(&Shutdown { error })?, StatusCode::OK))
}
