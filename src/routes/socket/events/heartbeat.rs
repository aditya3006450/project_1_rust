use axum::extract::ws::Message;
use tokio::sync::mpsc;

use crate::{
    app_state::AppState,
    routes::socket::types::{HeartbeatMessage, SocketMessage},
};
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn handle_heartbeat(
    _message: SocketMessage,
    _state: AppState,
    tx: &mpsc::Sender<Message>,
) -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let response = HeartbeatMessage {
        event: "pong".to_string(),
        timestamp: now,
    };

    let _ = tx.send(Message::Text(
        serde_json::to_string(&response).unwrap_or_default().into()
    )).await;

    now
}
