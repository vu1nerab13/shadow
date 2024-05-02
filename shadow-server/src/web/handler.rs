use crate::{
    network::{ServerObj, SystemPowerAction},
    web::error,
};
use anyhow::Result as AppResult;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, str::FromStr, sync::Arc};
use strum_macros::EnumString;
use tokio::sync::RwLock;
use warp::{http::StatusCode, reply, Rejection, Reply};

type Response<T> = Result<T, Rejection>;

#[derive(EnumString)]
enum ClientOperation {
    #[strum(ascii_case_insensitive)]
    Summary,
    #[strum(ascii_case_insensitive)]
    Shutdown,
    #[strum(ascii_case_insensitive)]
    Reboot,
    #[strum(ascii_case_insensitive)]
    Sleep,
    #[strum(ascii_case_insensitive)]
    Logout,
    #[strum(ascii_case_insensitive)]
    Hibernate,
    #[strum(ascii_case_insensitive)]
    Disconnect,
}

#[derive(EnumString)]
enum ServerOperation {
    #[strum(ascii_case_insensitive)]
    Query,
}

/// The route which handles all client request
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

    let (reply, status) = match match ServerOperation::from_str(&addr) {
        Ok(o) => match o {
            ServerOperation::Query => query_clients(server_objs, op).await,
        },
        Err(_) => try_client_op(addr, op, server_objs).await,
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

/// Query all clients that connected to the server
async fn query_clients(
    server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
    _op: Option<String>,
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

/// Try to perform a operation on a specific client
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
        Some(o) => ClientOperation::from_str(&o)?,
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
        ClientOperation::Summary => get_client_summary(server_obj).await,
        ClientOperation::Disconnect => get_client_disconnect(server_obj).await,
        ClientOperation::Shutdown => {
            get_client_power(server_obj, SystemPowerAction::Shutdown).await
        }
        ClientOperation::Reboot => get_client_power(server_obj, SystemPowerAction::Reboot).await,
        ClientOperation::Sleep => get_client_power(server_obj, SystemPowerAction::Sleep).await,
        ClientOperation::Logout => get_client_power(server_obj, SystemPowerAction::Logout).await,
        ClientOperation::Hibernate => {
            get_client_power(server_obj, SystemPowerAction::Hibernate).await
        }
    }
}

/// Get a client's summary
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

/// Let a client to shutdown
async fn get_client_power(
    server_obj: &Arc<RwLock<ServerObj>>,
    action: SystemPowerAction,
) -> AppResult<(String, StatusCode)> {
    #[derive(Serialize, Deserialize)]
    struct Shutdown {
        error: error::WebError,
    }

    let error = match server_obj.read().await.system_power(action).await? {
        true => error::WebError::Success,
        false => error::WebError::UnknownError,
    };

    Ok((serde_json::to_string(&Shutdown { error })?, StatusCode::OK))
}

/// Disconnect a client
async fn get_client_disconnect(
    server_obj: &Arc<RwLock<ServerObj>>,
) -> AppResult<(String, StatusCode)> {
    #[derive(Serialize, Deserialize)]
    struct Shutdown {
        error: error::WebError,
    }

    let error = match server_obj.read().await.disconnect() {
        true => error::WebError::Success,
        false => error::WebError::UnknownError,
    };

    Ok((serde_json::to_string(&Shutdown { error })?, StatusCode::OK))
}
