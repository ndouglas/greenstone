use super::Client;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type Clients = Arc<RwLock<HashMap<String, Client>>>;

lazy_static! {
  pub static ref CLIENTS: Clients = Arc::new(RwLock::new(HashMap::new()));
}
