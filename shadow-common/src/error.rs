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
}

impl From<CallError> for ShadowError {
    fn from(err: CallError) -> Self {
        Self::CallError(err)
    }
}
