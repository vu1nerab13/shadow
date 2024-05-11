use super::error;
use crate::network::{ServerObj, SystemPowerAction};
use anyhow::Result as AppResult;
use serde::{Deserialize, Serialize};
use shadow_common::client as sc;
use std::{collections::HashMap, net::SocketAddr, str::FromStr, sync::Arc};
use strum_macros::EnumString;
use tokio::sync::RwLock;
use warp::{http::StatusCode, reply, Rejection, Reply};

type Response<T> = Result<T, Rejection>;

#[derive(Deserialize, Serialize)]
pub struct ClientParam {
    op: String,
    addr: String,
}

#[derive(Deserialize, Serialize)]
pub struct ServerParam {
    op: String,
}

#[derive(EnumString, Deserialize, Serialize)]
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
    GetApps,
    #[strum(ascii_case_insensitive)]
    GetProcesses,
    #[strum(ascii_case_insensitive)]
    Disconnect,
}

#[derive(EnumString, Deserialize, Serialize)]
enum ServerOperation {
    #[strum(ascii_case_insensitive)]
    Query,
}

/// The route which handles all client request
pub async fn client_request(
    param: ClientParam,
    server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
) -> Response<Box<dyn Reply>> {
    #[derive(Serialize, Deserialize)]
    struct DispatchError {
        message: String,
        error: error::WebError,
    }

    let addr = match param.addr.parse() {
        Ok(a) => a,
        Err(_) => {
            return Ok(Box::new(reply::with_status(
                reply::json(&DispatchError {
                    message: param.addr,
                    error: error::WebError::AddressInvalid,
                }),
                StatusCode::BAD_REQUEST,
            )))
        }
    };

    let lock = server_objs.read().await;
    let server_obj = match lock.get(&addr) {
        Some(o) => o,
        None => {
            return Ok(Box::new(reply::with_status(
                reply::json(&DispatchError {
                    message: addr.to_string(),
                    error: error::WebError::ClientNotFound,
                }),
                StatusCode::NOT_FOUND,
            )))
        }
    };

    let reply = match ClientOperation::from_str(&param.op) {
        Ok(op) => match client_op(op, server_obj).await {
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

/// Try to perform a operation on a specific client
async fn client_op(
    op: ClientOperation,
    server_obj: &Arc<RwLock<ServerObj>>,
) -> AppResult<Box<dyn Reply>> {
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
        ClientOperation::GetApps => get_client_apps(server_obj).await,
        ClientOperation::GetProcesses => get_client_processes(server_obj).await,
    }
}

/// Get a client's summary
async fn get_client_summary(server_obj: &Arc<RwLock<ServerObj>>) -> AppResult<Box<dyn Reply>> {
    #[derive(Serialize, Deserialize)]
    struct GetSummary {
        summary: String,
    }

    Ok(Box::new(reply::with_status(
        reply::json(&GetSummary {
            summary: server_obj.read().await.summary(),
        }),
        StatusCode::OK,
    )))
}

/// Let a client to shutdown
async fn get_client_power(
    server_obj: &Arc<RwLock<ServerObj>>,
    action: SystemPowerAction,
) -> AppResult<Box<dyn Reply>> {
    #[derive(Serialize, Deserialize)]
    struct Shutdown {
        error: error::WebError,
    }

    let error = match server_obj.read().await.system_power(action).await? {
        true => error::WebError::Success,
        false => error::WebError::UnknownError,
    };

    Ok(Box::new(reply::with_status(
        reply::json(&Shutdown { error }),
        StatusCode::OK,
    )))
}

/// Get client's apps
async fn get_client_apps(server_obj: &Arc<RwLock<ServerObj>>) -> AppResult<Box<dyn Reply>> {
    #[derive(Serialize, Deserialize)]
    struct GetApps {
        apps: Vec<sc::App>,
    }

    Ok(Box::new(reply::with_status(
        reply::json(&GetApps {
            apps: server_obj.read().await.get_installed_apps().await?,
        }),
        StatusCode::OK,
    )))
}

/// Get client's processes
async fn get_client_processes(server_obj: &Arc<RwLock<ServerObj>>) -> AppResult<Box<dyn Reply>> {
    #[derive(Serialize, Deserialize)]
    struct GetProcesses {
        processes: Vec<sc::Process>,
    }

    Ok(Box::new(reply::with_status(
        reply::json(&GetProcesses {
            processes: server_obj.read().await.get_processes().await?,
        }),
        StatusCode::OK,
    )))
}

/// Disconnect a client
async fn get_client_disconnect(server_obj: &Arc<RwLock<ServerObj>>) -> AppResult<Box<dyn Reply>> {
    #[derive(Serialize, Deserialize)]
    struct Shutdown {
        error: error::WebError,
    }

    let error = match server_obj.read().await.disconnect() {
        true => error::WebError::Success,
        false => error::WebError::UnknownError,
    };

    Ok(Box::new(reply::with_status(
        reply::json(&Shutdown { error }),
        StatusCode::OK,
    )))
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
