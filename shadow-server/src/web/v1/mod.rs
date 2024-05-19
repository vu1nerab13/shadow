pub mod error;
mod handler;

use crate::network::ServerObj;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use warp::{
    filters::{query, BoxedFilter},
    path,
    reject::Rejection,
    reply::Reply,
    Filter,
};

// pub fn setup_routes(
//     server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
// ) -> BoxedFilter<(impl Reply,) {
//     let server_objs_c = server_objs.clone();
//     let v1_client = warp::path("v1")
//         .and(warp::path("client"))
//         .and(path::end())
//         .and(query::query::<ClientParam>())
//         .and_then(move |param: ClientParam| client_request(param, server_objs_c.clone()));

//     let server_objs_s = server_objs.clone();
//     let v1_server = warp::path("v1")
//         .and(warp::path("server"))
//         .and(path::end())
//         .and(query::query::<ServerParam>())
//         .and_then(move |param: ServerParam| server_request(param, server_objs_s.clone()));

//     v1_client
//         .or(v1_server)
//         .with(warp::cors().allow_any_origin())
// }

pub fn setup_routes(
    server_objs: Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<ServerObj>>>>>,
) -> BoxedFilter<(impl Reply,)> {
    let server = handler::server::setup_routes(server_objs.clone());
    let client = handler::client::setup_routes(server_objs);

    server.or(client).boxed()
}
