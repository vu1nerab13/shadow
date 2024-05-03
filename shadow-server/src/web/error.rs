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

    #[error("unknown error")]
    UnknownError,
}
