use anyhow::Result as AppResult;
use flexi_logger::Logger;
use shadow_server::{network, web};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> AppResult<()> {
    let server_objs = Arc::new(RwLock::new(HashMap::new()));

    Logger::try_with_str("trace")?.start()?;
    let server = tokio::spawn(network::run(server_objs.clone()));
    tokio::spawn(web::run(server_objs));

    server.await?
}
