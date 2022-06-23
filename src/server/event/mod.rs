use super::Topic;

pub struct Event {
  pub topic: Topic,
  pub user_id: Option<usize>,
  pub message: String,
}
