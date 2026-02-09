use axum::extract::ws::Message;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};

use crate::{
    app_state::AppState,
    routes::socket::{
        redis_manager::publish_message,
        types::{ErrorResponse, RedisMessage, SocketMessage},
    },
};

// Track pending messages waiting for delivery confirmation
pub type PendingMessages = Arc<Mutex<HashMap<String, mpsc::Sender<bool>>>>;

pub async fn forward_to_peer(
    message: SocketMessage,
    state: AppState,
    tx: &mpsc::Sender<Message>,
    pending_messages: &PendingMessages,
) {
    // First, try to find locally
    let local_found = {
        let email_device_map = state.email_device_to_socket.read().await;

        if let Some(device_map) = email_device_map.get(&message.to_email) {
            if let Some(socket_id) = device_map.get(&message.to_device) {
                let socket_connections = state.socket_id_to_connection.read().await;

                if let Some(target_tx) = socket_connections.get(socket_id) {
                    let msg_text = serde_json::to_string(&message).unwrap_or_default();
                    let _ = target_tx.send(Message::Text(msg_text.into())).await;
                    true
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    };

    if local_found {
        return;
    }

    // If not found locally, publish to Redis for other pods
    let message_id = format!("{}_{}_{}_{}_{}", 
        message.from_email, 
        message.from_device,
        message.to_email,
        message.to_device,
        chrono::Utc::now().timestamp_millis()
    );

    let redis_message = RedisMessage {
        target_email: message.to_email.clone(),
        target_device: message.to_device.clone(),
        socket_message: message.clone(),
        sender_pod: None,
        timestamp: Some(chrono::Utc::now().timestamp_millis() as u64),
    };

    // Create channel for delivery confirmation
    let (confirm_tx, mut confirm_rx) = mpsc::channel::<bool>(1);
    {
        let mut pending = pending_messages.lock().await;
        pending.insert(message_id.clone(), confirm_tx);
    }

    if let Err(e) = publish_message(&state, &redis_message).await {
        eprintln!("Failed to publish message to Redis: {}", e);
        // Remove from pending and send error
        let mut pending = pending_messages.lock().await;
        pending.remove(&message_id);
        
        let error_response = ErrorResponse {
            event: "error".to_string(),
            error: "Failed to route message - Redis unavailable".to_string(),
            target_email: Some(message.to_email.clone()),
            target_device: Some(message.to_device.clone()),
        };
        let _ = tx.send(Message::Text(
            serde_json::to_string(&error_response).unwrap_or_default().into()
        )).await;
        return;
    }

    // Wait for delivery confirmation or timeout
    let timeout = tokio::time::timeout(Duration::from_secs(5), confirm_rx.recv()).await;
    
    // Clean up pending
    let mut pending = pending_messages.lock().await;
    pending.remove(&message_id);

    match timeout {
        Ok(Some(true)) => {
            // Message was delivered successfully on another pod
            // No action needed, target received it
        }
        _ => {
            // Timeout or no confirmation - target not found
            let error_response = ErrorResponse {
                event: "target_not_found".to_string(),
                error: format!("User {} with device {} is not online", message.to_email, message.to_device),
                target_email: Some(message.to_email.clone()),
                target_device: Some(message.to_device.clone()),
            };
            let _ = tx.send(Message::Text(
                serde_json::to_string(&error_response).unwrap_or_default().into()
            )).await;
        }
    }
}

pub async fn confirm_message_delivery(
    pending_messages: &PendingMessages,
    message_id: String,
    delivered: bool,
) {
    let mut pending = pending_messages.lock().await;
    if let Some(confirm_tx) = pending.remove(&message_id) {
        let _ = confirm_tx.send(delivered).await;
    }
}
