use super::super::error::{self, Error};
use crate::network::ServerObj;
use anyhow::Result as AppResult;
use serde::{Deserialize, Serialize};
use shadow_common::client::SystemPowerAction;
use std::{collections::HashMap, net::SocketAddr, str::FromStr, sync::Arc};
use strum_macros::EnumString;
use tokio::sync::RwLock;
use warp::{
    filters::{any, body, query, BoxedFilter},
    http::StatusCode,
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
    #[strum(ascii_case_insensitive)]
    Displays,
}

#[derive(EnumString, Deserialize, Serialize)]
enum FileOperation {
    #[strum(ascii_case_insensitive)]
    Enumerate,
    #[strum(ascii_case_insensitive)]
    Read,
    #[strum(ascii_case_insensitive)]
    Create,
    #[strum(ascii_case_insensitive)]
    Write,
    #[strum(ascii_case_insensitive)]
    DeleteFile,
    #[strum(ascii_case_insensitive)]
    DeleteDir,
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
        server_obj: Arc<RwLock<ServerObj>>,
    ) -> Response<Box<dyn Reply>> {
        match op {
            QueryOperation::Summary => summarize_client(server_obj).await,
            QueryOperation::Apps => get_client_apps(server_obj).await,
            QueryOperation::Processes => get_client_processes(server_obj).await,
            QueryOperation::Displays => get_client_displays(server_obj).await,
        }
    }
}

#[derive(Deserialize, Serialize)]
struct PowerParameter {
    op: String,
}

impl Parameter for PowerParameter {
    type Operation = SystemPowerAction;

    fn operation(&self) -> AppResult<Self::Operation> {
        Ok(Self::Operation::from_str(&self.op)?)
    }

    async fn dispatch(
        &self,
        op: Self::Operation,
        server_obj: Arc<RwLock<ServerObj>>,
    ) -> Response<Box<dyn Reply>> {
        let (message, error, code) = match server_obj.read().await.system_power(op).await {
            Ok(b) => match b {
                true => ("".into(), error::WebError::Success, StatusCode::OK),
                false => (
                    "".into(),
                    error::WebError::UnknownError,
                    StatusCode::BAD_REQUEST,
                ),
            },
            Err(e) => (
                e.to_string(),
                error::WebError::UnknownError,
                StatusCode::BAD_REQUEST,
            ),
        };

        return Ok(Box::new(reply::with_status(
            reply::json(&Error { message, error }),
            code,
        )));
    }
}

#[derive(Deserialize, Serialize)]
struct FileParameter {
    op: String,
    path: String,
    #[serde(with = "serde_bytes", default)]
    content: Option<Vec<u8>>,
    #[serde(default)]
    dir: Option<bool>,
}

impl Parameter for FileParameter {
    type Operation = FileOperation;

    fn operation(&self) -> AppResult<Self::Operation> {
        Ok(Self::Operation::from_str(&self.op)?)
    }

    async fn dispatch(
        &self,
        op: Self::Operation,
        server_obj: Arc<RwLock<ServerObj>>,
    ) -> Response<Box<dyn Reply>> {
        match op {
            FileOperation::Enumerate => enumerate_directory(server_obj, &self.path).await,
            FileOperation::Read => read_file(server_obj, &self.path).await,
            FileOperation::Create => create(server_obj, &self.path, &self.dir).await,
            FileOperation::Write => write_file(server_obj, &self.path, &self.content).await,
            FileOperation::DeleteFile => delete_file(server_obj, &self.path).await,
            FileOperation::DeleteDir => delete_dir_recursive(server_obj, &self.path).await,
        }
    }
}

trait Parameter {
    type Operation;

    fn operation(&self) -> AppResult<Self::Operation>;

    async fn dispatch(
        &self,
        op: Self::Operation,
        server_obj: Arc<RwLock<ServerObj>>,
    ) -> Response<Box<dyn Reply>>;

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

        self.dispatch(op, server_obj).await
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
        .and(query::query::<QueryParameter>())
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

async fn summarize_client(server_obj: Arc<RwLock<ServerObj>>) -> Response<Box<dyn Reply>> {
    Ok(Box::new(reply::with_status(
        reply::json(&server_obj.read().await.summary()),
        StatusCode::OK,
    )))
}

async fn get_client_apps(server_obj: Arc<RwLock<ServerObj>>) -> Response<Box<dyn Reply>> {
    let apps = match server_obj.read().await.get_installed_apps().await {
        Ok(a) => a,
        Err(e) => {
            return Ok(Box::new(reply::with_status(
                reply::json(&Error {
                    message: e.to_string(),
                    error: error::WebError::UnknownError,
                }),
                StatusCode::BAD_REQUEST,
            )))
        }
    };

    Ok(Box::new(reply::with_status(
        reply::json(&apps),
        StatusCode::OK,
    )))
}

async fn get_client_processes(server_obj: Arc<RwLock<ServerObj>>) -> Response<Box<dyn Reply>> {
    let processes = match server_obj.read().await.get_processes().await {
        Ok(p) => p,
        Err(e) => {
            return Ok(Box::new(reply::with_status(
                reply::json(&Error {
                    message: e.to_string(),
                    error: error::WebError::UnknownError,
                }),
                StatusCode::BAD_REQUEST,
            )))
        }
    };

    Ok(Box::new(reply::with_status(
        reply::json(&processes),
        StatusCode::OK,
    )))
}

async fn get_client_displays(server_obj: Arc<RwLock<ServerObj>>) -> Response<Box<dyn Reply>> {
    let displays = match server_obj.read().await.get_displays().await {
        Ok(p) => p,
        Err(e) => {
            return Ok(Box::new(reply::with_status(
                reply::json(&Error {
                    message: e.to_string(),
                    error: error::WebError::UnknownError,
                }),
                StatusCode::BAD_REQUEST,
            )))
        }
    };

    Ok(Box::new(reply::with_status(
        reply::json(&displays),
        StatusCode::OK,
    )))
}

async fn enumerate_directory(
    server_obj: Arc<RwLock<ServerObj>>,
    path: &String,
) -> Response<Box<dyn Reply>> {
    let files = match server_obj.read().await.get_file_list(path).await {
        Ok(f) => f,
        Err(e) => {
            return Ok(Box::new(reply::with_status(
                reply::json(&Error {
                    message: e.to_string(),
                    error: error::WebError::UnknownError,
                }),
                StatusCode::BAD_REQUEST,
            )))
        }
    };

    Ok(Box::new(reply::with_status(
        reply::json(&files),
        StatusCode::OK,
    )))
}

async fn read_file(server_obj: Arc<RwLock<ServerObj>>, path: &String) -> Response<Box<dyn Reply>> {
    let files = match server_obj.read().await.get_file_content(path).await {
        Ok(f) => f,
        Err(e) => {
            return Ok(Box::new(reply::with_status(
                reply::json(&Error {
                    message: e.to_string(),
                    error: error::WebError::UnknownError,
                }),
                StatusCode::BAD_REQUEST,
            )))
        }
    };

    Ok(Box::new(reply::with_status(files, StatusCode::OK)))
}

async fn create(
    server_obj: Arc<RwLock<ServerObj>>,
    path: &String,
    dir: &Option<bool>,
) -> Response<Box<dyn Reply>> {
    let dir = dir.unwrap_or(false);

    match match dir {
        true => server_obj.read().await.create_dir(path).await,
        false => server_obj.read().await.create_file(path).await,
    } {
        Ok(f) => f,
        Err(e) => {
            return Ok(Box::new(reply::with_status(
                reply::json(&Error {
                    message: e.to_string(),
                    error: error::WebError::UnknownError,
                }),
                StatusCode::BAD_REQUEST,
            )))
        }
    };

    Ok(Box::new(reply::with_status(
        reply::json(&Error {
            message: "".into(),
            error: error::WebError::Success,
        }),
        StatusCode::OK,
    )))
}

async fn write_file(
    server_obj: Arc<RwLock<ServerObj>>,
    path: &String,
    content: &Option<Vec<u8>>,
) -> Response<Box<dyn Reply>> {
    let content = match content {
        Some(c) => c,
        None => {
            return Ok(Box::new(reply::with_status(
                reply::json(&Error {
                    message: "content not provided".into(),
                    error: error::WebError::ParamInvalid,
                }),
                StatusCode::BAD_REQUEST,
            )))
        }
    };

    match server_obj
        .read()
        .await
        .write_file(path, content.clone())
        .await
    {
        Ok(f) => f,
        Err(e) => {
            return Ok(Box::new(reply::with_status(
                reply::json(&Error {
                    message: e.to_string(),
                    error: error::WebError::UnknownError,
                }),
                StatusCode::BAD_REQUEST,
            )))
        }
    };

    Ok(Box::new(reply::with_status(
        reply::json(&Error {
            message: "".into(),
            error: error::WebError::Success,
        }),
        StatusCode::OK,
    )))
}

async fn delete_file(
    server_obj: Arc<RwLock<ServerObj>>,
    path: &String,
) -> Response<Box<dyn Reply>> {
    match server_obj.read().await.delete_file(path).await {
        Ok(f) => f,
        Err(e) => {
            return Ok(Box::new(reply::with_status(
                reply::json(&Error {
                    message: e.to_string(),
                    error: error::WebError::UnknownError,
                }),
                StatusCode::BAD_REQUEST,
            )))
        }
    };

    Ok(Box::new(reply::with_status(
        reply::json(&Error {
            message: "".into(),
            error: error::WebError::Success,
        }),
        StatusCode::OK,
    )))
}

async fn delete_dir_recursive(
    server_obj: Arc<RwLock<ServerObj>>,
    path: &String,
) -> Response<Box<dyn Reply>> {
    match server_obj.read().await.delete_dir_recursive(path).await {
        Ok(f) => f,
        Err(e) => {
            return Ok(Box::new(reply::with_status(
                reply::json(&Error {
                    message: e.to_string(),
                    error: error::WebError::UnknownError,
                }),
                StatusCode::BAD_REQUEST,
            )))
        }
    };

    Ok(Box::new(reply::with_status(
        reply::json(&Error {
            message: "".into(),
            error: error::WebError::Success,
        }),
        StatusCode::OK,
    )))
}
