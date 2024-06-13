use serde::{Deserialize, Serialize};

/// Web error message
#[derive(Serialize, Deserialize, Debug)]
pub struct Error {
    pub error: String,
    pub message: String,
}
