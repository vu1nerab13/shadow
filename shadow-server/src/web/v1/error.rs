use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Error {
    pub error: String,
    pub message: String,
}
