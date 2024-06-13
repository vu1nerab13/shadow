use anyhow::Result as AppResult;
use clap::Parser;
use flexi_logger::Logger;
use shadow_server::{network, web, AppArgs};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> AppResult<()> {
    let args = AppArgs::parse();

    // Start logging
    #[cfg(debug_assertions)]
    Logger::try_with_str(args.verbose)?.start()?;

    // A instance representing all clients connected to the server
    let server_objs = Arc::new(RwLock::new(HashMap::new()));
    // Server config
    let server_cfg = network::Config::new(args.server_addr.parse()?);
    // Web interface config
    let web_cfg = web::Config::new(args.web_addr.parse()?);

    // Start the server
    let server = tokio::spawn(network::run(server_cfg, server_objs.clone()));

    // Start web interface
    tokio::spawn(web::run(web_cfg, server_objs));

    // Wait until server shutdown
    server.await?
}
