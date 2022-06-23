use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct RegistrationRequest {
  pub user_id: usize,
}
