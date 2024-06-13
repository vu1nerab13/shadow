use crabgrab::{capturable_content::CapturableContentError, feature::screenshot::ScreenshotError};
use psutil::process::ProcessError;
use remoc::rtc::CallError;
use serde::{Deserialize, Serialize};
use std::io;
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum ShadowError {
    #[error("connect error, message: {0}")]
    CallError(String),

    #[error("system power error")]
    SystemPowerError,

    #[error("client not found")]
    ClientNotFound,

    #[error("can not list installed apps, message: {0}")]
    QueryAppsError(String),

    #[error("can not list directory: {0}, message: {1}")]
    QueryFilesError(String, String),

    #[error("operation not permitted")]
    AccessDenied,

    #[error("can not get capturable content, message: {0}")]
    GetCapturableContentError(String),

    #[error("no such display")]
    NoSuchDisplay,

    #[error("unsupported")]
    Unsupported,

    #[error("io error: {0}")]
    IoError(String),

    #[error("no private key")]
    NoPrivateKey,

    #[error("request successfully")]
    Success,

    #[error("no operation provided")]
    NoOp,

    #[error("address is invalid")]
    AddressInvalid,

    #[error("unknown error")]
    UnknownError,

    #[error("process not found, message: {0}")]
    ProcessNotFound(String),

    #[error("param is invalid, message: {0}")]
    ParamInvalid(String),

    #[error("failed to kill a process, message: {0}")]
    KillProcessError(String),
}

impl From<CallError> for ShadowError {
    fn from(err: CallError) -> Self {
        Self::CallError(err.to_string())
    }
}

impl From<CapturableContentError> for ShadowError {
    fn from(err: CapturableContentError) -> Self {
        Self::GetCapturableContentError(err.to_string())
    }
}

impl From<io::Error> for ShadowError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}

impl From<ScreenshotError> for ShadowError {
    fn from(err: ScreenshotError) -> Self {
        Self::IoError(err.to_string())
    }
}

impl From<ProcessError> for ShadowError {
    fn from(err: ProcessError) -> Self {
        Self::KillProcessError(err.to_string())
    }
}
