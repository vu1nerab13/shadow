use remoc::rtc::CallError;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug, serde::Serialize, serde::Deserialize)]
pub enum ShadowError {
    #[error("the client for uuid `{0}` is not available")]
    ClientNotExist(Uuid),

    #[error("connect error")]
    CallError(CallError),
}

impl From<CallError> for ShadowError {
    fn from(err: CallError) -> Self {
        Self::CallError(err)
    }
}
