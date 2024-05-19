use crate::network::ServerObj;
use crate::web::v1;
use anyhow::Result;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use warp::{filters::query, path, Filter};

pub struct Config {
    addr: SocketAddr,
}

impl Config {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }
}

pub async fn run(
    cfg: Config,
    server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
) -> Result<()> {
    // Root page
    let root = path::end().map(|| "Welcome to shadow server!");

    let v1_api = v1::setup_routes(server_objs.clone());

    warp::serve(root.or(v1_api)).run(cfg.addr).await;

    Ok(())
}
