use super::*;
use std::result::Result;
use tokio::sync::mpsc::UnboundedSender;
use warp::ws::Message;
use warp::Error;

/// Represents the user accessing the server via WebSocket.
pub struct Client {
  pub user_id: usize,
  pub sender: Option<UnboundedSender<Result<Message, Error>>>,
  pub topics: Vec<Topic>,
}
