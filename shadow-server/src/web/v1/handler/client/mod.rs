use super::super::error::{self, Error};
use crate::network::ServerObj;
use serde::{Deserialize, Serialize};
use shadow_common::client::{App, Process, SystemInfo};
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
    Summary,
    #[strum(ascii_case_insensitive)]
    Apps,
    #[strum(ascii_case_insensitive)]
    Processes,
}

#[derive(Deserialize, Serialize)]
struct QueryParameter {
    op: String,
}

pub fn setup_routes(
    server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
) -> BoxedFilter<(impl Reply,)> {
    let prefix =
        path!("v1" / "client" / SocketAddr / ..).and(any::any().map(move || server_objs.clone()));

    let query = prefix
        .and(warp::get())
        .and(path!("query"))
        .and(path::end())
        .and(query::query())
        .and_then(query);

    query.boxed()
}

async fn query(
    addr: SocketAddr,
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

    let lock = server_objs.read().await;
    let server_obj = match lock.get(&addr) {
        Some(o) => o,
        None => {
            return Ok(Box::new(reply::with_status(
                reply::json(&Error {
                    message: addr.to_string(),
                    error: error::WebError::ClientNotFound,
                }),
                StatusCode::OK,
            )))
        }
    };

    match op {
        QueryOperation::Summary => summarize_client(server_obj).await,
        QueryOperation::Apps => get_client_apps(server_obj).await,
        QueryOperation::Processes => get_client_processes(server_obj).await,
    }
}

async fn summarize_client(server_obj: &Arc<RwLock<ServerObj>>) -> Response<Box<dyn Reply>> {
    #[derive(Serialize, Deserialize)]
    struct Summary {
        summary: SystemInfo,
    }

    Ok(Box::new(reply::with_status(
        reply::json(&Summary {
            summary: server_obj.read().await.summary(),
        }),
        StatusCode::OK,
    )))
}

async fn get_client_apps(server_obj: &Arc<RwLock<ServerObj>>) -> Response<Box<dyn Reply>> {
    #[derive(Serialize, Deserialize)]
    struct Apps {
        apps: Vec<App>,
    }

    let apps = match server_obj.read().await.get_installed_apps().await {
        Ok(a) => a,
        Err(e) => {
            return Ok(Box::new(reply::with_status(
                reply::json(&Error {
                    message: e.to_string(),
                    error: error::WebError::UnknownError,
                }),
                StatusCode::OK,
            )))
        }
    };

    Ok(Box::new(reply::with_status(
        reply::json(&Apps { apps }),
        StatusCode::OK,
    )))
}

async fn get_client_processes(server_obj: &Arc<RwLock<ServerObj>>) -> Response<Box<dyn Reply>> {
    #[derive(Serialize, Deserialize)]
    struct Processes {
        processes: Vec<Process>,
    }

    let processes = match server_obj.read().await.get_processes().await {
        Ok(p) => p,
        Err(e) => {
            return Ok(Box::new(reply::with_status(
                reply::json(&Error {
                    message: e.to_string(),
                    error: error::WebError::UnknownError,
                }),
                StatusCode::OK,
            )))
        }
    };

    Ok(Box::new(reply::with_status(
        reply::json(&Processes { processes }),
        StatusCode::OK,
    )))
}
