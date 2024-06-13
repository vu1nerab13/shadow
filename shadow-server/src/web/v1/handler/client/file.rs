use super::Parameter;
use crate::network::ServerObj;
use anyhow::Result as AppResult;
use serde::{Deserialize, Serialize};
use shadow_common::error::ShadowError;
use std::{str::FromStr, sync::Arc};
use strum_macros::EnumString;
use tokio::sync::RwLock;
use warp::{
    http::StatusCode,
    reply::{self, Reply},
};

#[derive(EnumString, Deserialize, Serialize)]
pub enum FileOperation {
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
pub struct FileParameter {
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

    fn summarize() -> String {
        "file operation".into()
    }

    async fn dispatch(
        &self,
        op: Self::Operation,
        server_obj: Arc<RwLock<ServerObj>>,
    ) -> Result<Box<dyn Reply>, ShadowError> {
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

async fn enumerate_directory(
    server_obj: Arc<RwLock<ServerObj>>,
    path: &String,
) -> Result<Box<dyn Reply>, ShadowError> {
    let files = server_obj.read().await.get_file_list(path).await?;

    Ok(Box::new(reply::with_status(
        reply::json(&files),
        StatusCode::OK,
    )))
}

async fn read_file(
    server_obj: Arc<RwLock<ServerObj>>,
    path: &String,
) -> Result<Box<dyn Reply>, ShadowError> {
    let files = server_obj.read().await.get_file_content(path).await?;

    Ok(Box::new(reply::with_status(files, StatusCode::OK)))
}

async fn create(
    server_obj: Arc<RwLock<ServerObj>>,
    path: &String,
    dir: &Option<bool>,
) -> Result<Box<dyn Reply>, ShadowError> {
    let dir = dir.unwrap_or(false);

    match dir {
        true => server_obj.read().await.create_dir(path).await,
        false => server_obj.read().await.create_file(path).await,
    }?;

    super::success()
}

async fn write_file(
    server_obj: Arc<RwLock<ServerObj>>,
    path: &String,
    content: &Option<Vec<u8>>,
) -> Result<Box<dyn Reply>, ShadowError> {
    let content = super::require(content.clone(), "file content")?;

    server_obj.read().await.write_file(path, content).await?;

    super::success()
}

async fn delete_file(
    server_obj: Arc<RwLock<ServerObj>>,
    path: &String,
) -> Result<Box<dyn Reply>, ShadowError> {
    server_obj.read().await.delete_file(path).await?;

    super::success()
}

async fn delete_dir_recursive(
    server_obj: Arc<RwLock<ServerObj>>,
    path: &String,
) -> Result<Box<dyn Reply>, ShadowError> {
    server_obj.read().await.delete_dir_recursive(path).await?;

    super::success()
}
