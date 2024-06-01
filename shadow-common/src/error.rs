use crabgrab::capturable_content::CapturableContentError;
use remoc::rtc::CallError;
use thiserror::Error;

#[derive(Error, Debug, serde::Serialize, serde::Deserialize)]
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
