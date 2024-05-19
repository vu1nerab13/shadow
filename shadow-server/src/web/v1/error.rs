use serde::{Deserialize, Serialize};
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
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Error {
    pub error: WebError,
    pub message: String,
}
