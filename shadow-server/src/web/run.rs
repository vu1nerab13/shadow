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

    // V1 api
    let server_objs_c = server_objs.clone();
    let v1_client = warp::path("v1")
        .and(warp::path("client"))
        .and(path::end())
        .and(query::query::<v1::ClientParam>())
        .and_then(move |param: v1::ClientParam| v1::client_request(param, server_objs_c.clone()));

    let server_objs_s = server_objs.clone();
    let v1_server = warp::path("v1")
        .and(warp::path("server"))
        .and(path::end())
        .and(query::query::<v1::ServerParam>())
        .and_then(move |param: v1::ServerParam| v1::server_request(param, server_objs_s.clone()));

    let routes = root
        .or(v1_client)
        .or(v1_server)
        .with(warp::cors().allow_any_origin());

    warp::serve(routes).run(cfg.addr).await;

    Ok(())
}
