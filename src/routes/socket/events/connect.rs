use axum::extract::ws::Message;
use tokio::sync::mpsc;

use crate::{
    app_state::AppState,
    routes::socket::types::SocketMessage,
};

pub async fn on_connect(
    _message: SocketMessage,
    _state: AppState,
    tx: &mpsc::Sender<Message>,
) {
    // Acknowledge connection
    let response = Message::Text(
        serde_json::json!({
            "event": "connected",
            "status": "ok"
        })
        .to_string()
        .into(),
    );
    let _ = tx.send(response).await;
}
