use super::super::*;
use warp::http::StatusCode;
use warp::Reply;

pub async fn deregistration_handler(id: String, clients: Clients) -> Result<impl Reply> {
  clients.write().await.remove(&id);
  Ok(StatusCode::OK)
}
