use super::super::*;
use uuid::Uuid;
use warp::Reply;

pub async fn registration_handler(body: RegistrationRequest, clients: Clients) -> Result<impl Reply> {
  let user_id = body.user_id;
  let server = "127.0.0.1:44553";
  let uuid = Uuid::new_v4().simple().to_string();

  register_client(uuid.clone(), user_id, clients).await;
  Ok(warp::reply::json(&RegistrationResponse {
    url: format!("ws://{server}/ws/{uuid}"),
  }))
}

async fn register_client(id: String, user_id: usize, clients: Clients) {
  clients.write().await.insert(
    id,
    Client {
      user_id,
      topics: vec![Topic::Default],
      sender: None,
    },
  );
}
