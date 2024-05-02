use anyhow::Result as AppResult;
use flexi_logger::Logger;
use shadow_server::{network, web};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> AppResult<()> {
    Logger::try_with_str("trace")?.start()?;

    let server_objs = Arc::new(RwLock::new(HashMap::new()));
    let server_cfg = network::Config::new("0.0.0.0:1244".parse()?);
    let web_cfg = web::Config::new("0.0.0.0:5000".parse()?);

    let server = tokio::spawn(network::run(server_cfg, server_objs.clone()));

    tokio::spawn(web::run(web_cfg, server_objs));

    server.await?
}
