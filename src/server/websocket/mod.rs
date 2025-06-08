use super::*;
use futures::{FutureExt, StreamExt};
use serde_json::from_str;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};

pub async fn client_connection(ws: WebSocket, id: String, clients: Clients, mut client: Client) {
  let (client_ws_sender, mut client_ws_rcv) = ws.split();
  let (client_sender, client_rcv) = mpsc::unbounded_channel();
  let client_rcv = UnboundedReceiverStream::new(client_rcv);

  tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
    if let Err(e) = result {
      eprintln!("Error sending websocket message: {e}");
    }
  }));

  client.sender = Some(client_sender);
  clients.write().await.insert(id.clone(), client);

  println!("Websocket client {id} connected");

  while let Some(result) = client_ws_rcv.next().await {
    let msg = match result {
      Ok(msg) => msg,
      Err(e) => {
        eprintln!("Error receiving websocket message for id: {id}): {e}");
        break;
      }
    };
    client_msg(&id, msg, &clients).await;
  }

  clients.write().await.remove(&id);
  println!("Websocket client {id} disconnected");
}

async fn client_msg(id: &str, msg: Message, clients: &Clients) {
  println!("Received message from websocket client {id}: {msg:?}");
  let message = match msg.to_str() {
    Ok(v) => v,
    Err(_) => return,
  };

  if message == "ping" || message == "ping\n" {
    return;
  }

  let subscription_request: SubscriptionRequest = match from_str(message) {
    Ok(v) => v,
    Err(e) => {
      eprintln!("Error while parsing message to subscription request: {e}");
      return;
    }
  };

  let mut locked = clients.write().await;
  if let Some(v) = locked.get_mut(id) {
    v.topics = subscription_request.topics;
  }
}
