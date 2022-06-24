use super::super::*;
use warp::ws::Ws;
use warp::Reply;

pub async fn websocket_handler(ws: Ws, id: String, clients: Clients) -> Result<impl Reply> {
  let client = clients.read().await.get(&id).cloned();
  match client {
    Some(c) => Ok(ws.on_upgrade(move |socket| client_connection(socket, id, clients, c))),
    None => Err(warp::reject::not_found()),
  }
}
