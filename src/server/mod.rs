use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Filter;

pub mod client;
pub use client::*;

pub mod clients;
pub use clients::*;

pub mod event;
pub use event::*;

pub mod handlers;
pub use handlers::*;

pub mod request;
pub use request::*;

pub mod response;
pub use response::*;

pub mod result;
pub use result::*;

pub mod topic;
pub use topic::*;

pub mod websocket;
pub use websocket::*;

pub type Clients = Arc<RwLock<HashMap<String, Client>>>;

#[named]
pub async fn start_server() {
  trace_enter!();

  let health_route = warp::path!("health").and_then(health_handler);

  let register = warp::path("register");
  let registration_routes = register
    .and(warp::post())
    .and(warp::body::json())
    .and(with_clients(CLIENTS.clone()))
    .and_then(registration_handler)
    .or(
      register
        .and(warp::delete())
        .and(warp::path::param())
        .and(with_clients(CLIENTS.clone()))
        .and_then(deregistration_handler),
    );

  let websocket_route = warp::path("ws")
    .and(warp::ws())
    .and(warp::path::param())
    .and(with_clients(CLIENTS.clone()))
    .and_then(websocket_handler);

  let routes = health_route
    .or(registration_routes)
    .or(websocket_route)
    .with(warp::cors().allow_any_origin());

  warp::serve(routes).bind(([0, 0, 0, 0], 44553)).await;
  trace_exit!();
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
  warp::any().map(move || clients.clone())
}
