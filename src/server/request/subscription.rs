use super::super::Topic;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SubscriptionRequest {
  pub topics: Vec<Topic>,
}
