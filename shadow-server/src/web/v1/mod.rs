pub mod error;
mod handler;

use crate::network::ServerObj;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use warp::{filters::BoxedFilter, reply::Reply, Filter};

pub fn setup_routes(
    server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
) -> BoxedFilter<(impl Reply,)> {
    let server = handler::server::setup_routes(server_objs.clone());
    let client = handler::client::setup_routes(server_objs);

    server.or(client).boxed()
}
