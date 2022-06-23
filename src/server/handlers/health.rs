use super::super::Result;
use warp::http::StatusCode;
use warp::Reply;

pub async fn health_handler() -> Result<impl Reply> {
  Ok(StatusCode::OK)
}
