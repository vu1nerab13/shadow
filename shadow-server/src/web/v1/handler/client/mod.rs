use super::super::error::{self, Error};
use crate::network::ServerObj;
use serde::{Deserialize, Serialize};
use shadow_common::client::SystemPowerAction;
use std::{collections::HashMap, net::SocketAddr, str::FromStr, sync::Arc};
use strum_macros::EnumString;
use tokio::sync::RwLock;
use warp::{
    filters::{any, body, query, BoxedFilter},
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

#[derive(Deserialize, Serialize)]
struct PowerParameter {
    op: String,
}

pub fn setup_routes(
    server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
) -> BoxedFilter<(impl Reply,)> {
    let prefix =
        path!("v1" / "client" / SocketAddr / ..).and(any::any().map(move || server_objs.clone()));

    let query = prefix
        .clone()
        .and(warp::get())
        .and(path!("query"))
        .and(path::end())
        .and(query::query())
        .and_then(query);

    let power = prefix
        .and(warp::post())
        .and(path!("power"))
        .and(path::end())
        .and(body::json())
        .and_then(power);

    query.or(power).boxed()
}

async fn power(
    addr: SocketAddr,
    server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
    param: PowerParameter,
) -> Response<Box<dyn Reply>> {
    let op = match SystemPowerAction::from_str(&param.op) {
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
        Some(o) => o.clone(),
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

    let (message, error) = match server_obj.read().await.system_power(op).await {
        Ok(b) => match b {
            true => ("".into(), error::WebError::Success),
            false => ("".into(), error::WebError::UnknownError),
        },
        Err(e) => (e.to_string(), error::WebError::UnknownError),
    };

    return Ok(Box::new(reply::with_status(
        reply::json(&Error { message, error }),
        StatusCode::OK,
    )));
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
    Ok(Box::new(reply::with_status(
        reply::json(&server_obj.read().await.summary()),
        StatusCode::OK,
    )))
}

async fn get_client_apps(server_obj: &Arc<RwLock<ServerObj>>) -> Response<Box<dyn Reply>> {
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
        reply::json(&apps),
        StatusCode::OK,
    )))
}

async fn get_client_processes(server_obj: &Arc<RwLock<ServerObj>>) -> Response<Box<dyn Reply>> {
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
        reply::json(&processes),
        StatusCode::OK,
    )))
}
