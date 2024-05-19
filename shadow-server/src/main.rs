use anyhow::Result as AppResult;
use flexi_logger::Logger;
use shadow_server::{network, web};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> AppResult<()> {
    // Start logging
    #[cfg(debug_assertions)]
    Logger::try_with_str("debug")?.start()?;

    // A instance representing all clients connected to the server
    let server_objs = Arc::new(RwLock::new(HashMap::new()));
    // Server config
    let server_cfg = network::Config::new("0.0.0.0:1244".parse()?);
    // Web interface config
    let web_cfg = web::Config::new("0.0.0.0:9000".parse()?);

    // Start the server
    let server = tokio::spawn(network::run(server_cfg, server_objs.clone()));

    // Start web interface
    tokio::spawn(web::run(web_cfg, server_objs));

    // Wait until server shutdown
    server.await?
}
