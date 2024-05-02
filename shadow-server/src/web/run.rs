use crate::network::ServerObj;
use crate::web::handler;
use anyhow::Result;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use warp::{path, Filter};

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
    let optional = warp::path::param::<String>()
        .map(Some)
        .or_else(|_| async { Ok::<(Option<String>,), std::convert::Infallible>((None,)) });

    let client = warp::path("client")
        .and(path::param())
        .and(optional)
        .and(path::end())
        .and(warp::get())
        .and_then(move |addr: String, op: Option<String>| {
            handler::client_operation(addr, op, server_objs.clone())
        });

    let routes = root.or(client).with(warp::cors().allow_any_origin());

    warp::serve(routes).run(cfg.addr).await;

    Ok(())
}
