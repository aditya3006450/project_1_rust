use axum::{
    Router,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
    routing::get,
};
use futures_util::{SinkExt, StreamExt};
use std::ops::ControlFlow;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    routes::socket::{
        events::{
            check::check_users_response,
            connect::on_connect,
            disconnect::disconnect_user,
            forwarder::{PendingMessages, forward_to_peer},
            heartbeat::handle_heartbeat,
            register::register_user,
        },
        redis_manager::start_redis_subscriber,
        types::SocketMessage,
    },
};

pub fn ws_route(state: AppState) -> Router {
    start_redis_subscriber(state.clone());

    Router::new().route("/", get(ws_handler).with_state(state))
}

async fn ws_handler(State(state): State<AppState>, ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(move |socket| socket_handler(socket, state))
}

async fn socket_handler(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel(100);

    // Generate unique socket ID for this connection
    let socket_id = Uuid::new_v4().to_string();

    // Spawn task to send messages to the websocket
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let mut user_email: Option<String> = None;
    let mut device_id: Option<String> = None;
    let pending_messages: PendingMessages =
        Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));

    while let Some(Ok(msg)) = receiver.next().await {
        if process_message(
            msg,
            state.clone(),
            &tx,
            &mut user_email,
            &mut device_id,
            &socket_id,
            &pending_messages,
        )
        .await
        .is_break()
        {
            break;
        }
    }

    // Cleanup logic on disconnect
    if let (Some(email), Some(device)) = (user_email, device_id) {
        disconnect_user(email, device, socket_id, state).await;
    }
}

async fn process_message(
    msg: Message,
    state: AppState,
    tx: &mpsc::Sender<Message>,
    user_email: &mut Option<String>,
    device_id: &mut Option<String>,
    socket_id: &str,
    pending_messages: &PendingMessages,
) -> ControlFlow<(), ()> {
    match SocketMessage::parse_message(msg.clone()) {
        Ok(socket_message) => {
            // Validate the message
            if let Err(validation_error) = socket_message.validate() {
                let error_response = serde_json::json!({
                    "event": "error",
                    "error": validation_error
                });
                let _ = tx
                    .send(Message::Text(error_response.to_string().into()))
                    .await;
                return ControlFlow::Continue(());
            }

            match socket_message.event.as_str() {
                "register" => {
                    // Only allow registration if not already registered
                    // Clone necessary fields before moving socket_message
                    let from_email = socket_message.from_email.clone();
                    let from_device = socket_message.from_device.clone();

                    match register_user(socket_message, socket_id, state.clone()).await {
                        Ok(_) => {
                            *user_email = Some(from_email.clone());
                            *device_id = Some(from_device.clone());

                            // Store local mappings
                            let key = format!("{}{}", from_email, from_device);
                            state
                                .socket_connections
                                .write()
                                .await
                                .insert(key, tx.clone());

                            // Store in new mappings
                            state
                                .socket_id_to_connection
                                .write()
                                .await
                                .insert(socket_id.to_string(), tx.clone());

                            let mut email_device_map = state.email_device_to_socket.write().await;
                            if let Some(device_map) = email_device_map.get_mut(&from_email) {
                                device_map.insert(from_device.clone(), socket_id.to_string());
                            } else {
                                let mut device_map = std::collections::HashMap::new();
                                device_map.insert(from_device.clone(), socket_id.to_string());
                                email_device_map.insert(from_email, device_map);
                            }

                            // Send success response
                            let response = Message::Text(
                                serde_json::json!({
                                    "event": "register",
                                    "status": "ok",
                                    "socket_id": socket_id
                                })
                                .to_string()
                                .into(),
                            );
                            let _ = tx.send(response).await;
                        }
                        Err(e) => {
                            // Send error response
                            let response = Message::Text(
                                serde_json::json!({
                                    "event": "register",
                                    "status": "error",
                                    "error": e
                                })
                                .to_string()
                                .into(),
                            );
                            let _ = tx.send(response).await;
                            return ControlFlow::Break(());
                        }
                    }
                }
                "check" => {
                    if user_email.is_some() {
                        let users = check_users_response(
                            user_email.clone().unwrap_or_default(),
                            state.clone(),
                        )
                        .await;
                        let response =
                            Message::Text(serde_json::to_string(&users).unwrap_or_default().into());
                        if tx.send(response).await.is_err() {
                            return ControlFlow::Break(());
                        }
                    }
                }
                "connect" => {
                    if user_email.is_some() {
                        on_connect(socket_message, state.clone(), tx).await;
                    }
                }
                "ping" => {
                    if user_email.is_some() {
                        handle_heartbeat(socket_message, state.clone(), tx).await;
                    }
                }
                "try_connect" | "sdp_offer" | "sdp_answer" | "ice_candidate" => {
                    if user_email.is_some() {
                        forward_to_peer(socket_message, state.clone(), tx, pending_messages).await;
                    }
                }
                "disconnect" => {
                    return ControlFlow::Break(());
                }
                _ => {
                    let error_response = serde_json::json!({
                        "event": "error",
                        "error": format!("Unknown event: {}", socket_message.event)
                    });
                    let _ = tx
                        .send(Message::Text(error_response.to_string().into()))
                        .await;
                }
            }
        }
        Err(e) => {
            // Send parse error response
            let error_response = serde_json::json!({
                "event": "error",
                "error": format!("Failed to parse message: {}", e)
            });
            let _ = tx
                .send(Message::Text(error_response.to_string().into()))
                .await;
        }
    }
    ControlFlow::Continue(())
}
