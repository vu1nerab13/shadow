use crabgrab::{capturable_content::CapturableContentError, feature::screenshot::ScreenshotError};
use remoc::rtc::CallError;
use serde::{Deserialize, Serialize};
use std::io;
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum ShadowError {
    #[error("connect error")]
    CallError(CallError),

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

    #[error("can not get capturable content")]
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

    #[error("param is invalid")]
    ParamInvalid,
}

impl From<CallError> for ShadowError {
    fn from(err: CallError) -> Self {
        Self::CallError(err)
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
