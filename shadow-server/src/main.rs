use anyhow::Result as AppResult;
use flexi_logger::Logger;
use shadow_server::network;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> AppResult<()> {
    let server_objs = Arc::new(RwLock::new(HashMap::new()));

    Logger::try_with_str("trace")?.start()?;
    network::run(server_objs).await?;

    Ok(())
}
