use axum::{
    Router,
    extract::{State, WebSocketUpgrade, ws::WebSocket},
    response::IntoResponse,
    routing::get,
};

use crate::{
    app_state::AppState,
    routes::socket::{
        events::{ping::ping_users, register::register_user},
        types::SocketMessage,
    },
};

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
                    match socket_message.event.as_str() {
                        "connect" => (),
                        "ping" => {
                            ping_users(socket_message, state.clone());
                        }
                        "register" => {
                            register_user(socket_message, &mut socket, state.clone());
                        }
                        "try_connect" => (),
                        "disconnect" => (),
                        _ => (),
                    }
                } else {
                }
            }
            Err(ercr) => (),
        }
    }
}
