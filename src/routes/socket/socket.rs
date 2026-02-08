use axum::{
    Router,
    extract::{State, WebSocketUpgrade, ws::WebSocket},
    response::IntoResponse,
    routing::get,
};

use crate::{app_state::AppState, routes::socket::types::SocketMessage};

pub fn ws_route(state: AppState) -> Router {
    Router::new().route("/", get(ws_handler).with_state(state))
}

async fn ws_handler(State(state): State<AppState>, ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(move |socket| socket_handler(socket, state))
}

async fn socket_handler(mut socket: WebSocket, state: AppState) {
    while let Some(message) = socket.recv().await {
        match message {
            Ok(msg) => {
                if let Ok(socket_message) = SocketMessage::parse_message(msg.clone()) {
                } else {
                }
            }
            Err(ercr) => (),
        }
    }
}
