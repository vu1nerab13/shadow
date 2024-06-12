use serde::{Deserialize, Serialize};
use shadow_common::error::ShadowError;
use thiserror::Error;

#[derive(Serialize, Deserialize, Error, Debug)]
pub enum WebError {
    #[error("request successfully")]
    Success,

    #[error("client not found")]
    ClientNotFound,

    #[error("no operation provided")]
    NoOp,

    #[error("address is invalid")]
    AddressInvalid,

    #[error("unknown error")]
    UnknownError,

    #[error("param is invalid")]
    ParamInvalid,

    #[error("client encountered an error: {0}")]
    ClientError(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Error {
    pub error: WebError,
    pub message: String,
}
