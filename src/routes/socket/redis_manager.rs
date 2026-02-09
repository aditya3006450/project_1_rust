use axum::extract::ws::Message;
use futures_util::StreamExt;
use redis::AsyncCommands;
use serde_json;

use crate::{
    app_state::AppState,
    routes::socket::types::{DeviceInfo, RedisMessage},
};

#[derive(Debug)]
pub enum RedisManagerError {
    PoolError(String),
    SerializationError(String),
    RedisError(redis::RedisError),
}

impl std::fmt::Display for RedisManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RedisManagerError::PoolError(e) => write!(f, "Redis pool error: {}", e),
            RedisManagerError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            RedisManagerError::RedisError(e) => write!(f, "Redis error: {}", e),
        }
    }
}

impl From<redis::RedisError> for RedisManagerError {
    fn from(e: redis::RedisError) -> Self {
        RedisManagerError::RedisError(e)
    }
}

pub async fn publish_message(
    app_state: &AppState,
    message: &RedisMessage,
) -> Result<(), RedisManagerError> {
    let mut conn = app_state
        .redis_pool
        .get()
        .await
        .map_err(|e| RedisManagerError::PoolError(e.to_string()))?;

    let channel = "socket:messages";
    let message_json = serde_json::to_string(message)
        .map_err(|e| RedisManagerError::SerializationError(e.to_string()))?;

    let _: () = conn.publish(channel, message_json).await?;
    Ok(())
}

pub async fn store_device_presence(
    app_state: &AppState,
    email: &str,
    device_id: &str,
    device_info: &DeviceInfo,
) -> Result<(), RedisManagerError> {
    let mut conn = app_state
        .redis_pool
        .get()
        .await
        .map_err(|e| RedisManagerError::PoolError(e.to_string()))?;

    let presence_key = AppState::get_redis_presence_key(email, device_id);
    let user_devices_key = AppState::get_redis_user_devices_key(email);

    let device_info_json = serde_json::to_string(device_info)
        .map_err(|e| RedisManagerError::SerializationError(e.to_string()))?;

    // Store device presence
    let _: () = conn.set(&presence_key, &device_info_json).await?;

    // Add device to user's device set
    let _: () = conn
        .hset(&user_devices_key, device_id, &device_info.socket_id)
        .await?;

    Ok(())
}

pub async fn remove_device_presence(
    app_state: &AppState,
    email: &str,
    device_id: &str,
) -> Result<(), RedisManagerError> {
    let mut conn = app_state
        .redis_pool
        .get()
        .await
        .map_err(|e| RedisManagerError::PoolError(e.to_string()))?;

    let presence_key = AppState::get_redis_presence_key(email, device_id);
    let user_devices_key = AppState::get_redis_user_devices_key(email);

    // Remove device presence
    let _: () = conn.del(&presence_key).await?;

    // Remove device from user's device set
    let _: () = conn.hdel(&user_devices_key, device_id).await?;

    Ok(())
}

pub async fn get_user_devices(
    app_state: &AppState,
    email: &str,
) -> Result<Vec<DeviceInfo>, RedisManagerError> {
    let mut conn = app_state
        .redis_pool
        .get()
        .await
        .map_err(|e| RedisManagerError::PoolError(e.to_string()))?;

    let user_devices_key = AppState::get_redis_user_devices_key(email);

    // Get all device IDs for this user
    let device_ids: Vec<String> = conn.hkeys(&user_devices_key).await.unwrap_or_default();

    let mut devices = Vec::new();
    for device_id in device_ids {
        let presence_key = AppState::get_redis_presence_key(email, &device_id);
        if let Ok(device_info_json) = conn.get::<_, String>(&presence_key).await {
            if let Ok(device_info) = serde_json::from_str::<DeviceInfo>(&device_info_json) {
                devices.push(device_info);
            }
        }
    }

    Ok(devices)
}

pub async fn broadcast_user_joined(
    app_state: &AppState,
    email: &str,
    device_id: &str,
) -> Result<(), RedisManagerError> {
    let message = RedisMessage {
        target_email: "*".to_string(),
        target_device: "*".to_string(),
        socket_message: crate::routes::socket::types::SocketMessage {
            from_email: email.to_string(),
            from_token: String::new(),
            from_device: device_id.to_string(),
            to_email: String::new(),
            to_device: String::new(),
            event: "user_joined".to_string(),
            payload: serde_json::json!({"email": email, "device_id": device_id}),
        },
        sender_pod: None,
        timestamp: Some(chrono::Utc::now().timestamp_millis() as u64),
    };

    publish_message(app_state, &message).await
}

pub async fn broadcast_user_left(
    app_state: &AppState,
    email: &str,
    device_id: &str,
) -> Result<(), RedisManagerError> {
    let message = RedisMessage {
        target_email: "*".to_string(),
        target_device: "*".to_string(),
        socket_message: crate::routes::socket::types::SocketMessage {
            from_email: email.to_string(),
            from_token: String::new(),
            from_device: device_id.to_string(),
            to_email: String::new(),
            to_device: String::new(),
            event: "user_left".to_string(),
            payload: serde_json::json!({"email": email, "device_id": device_id}),
        },
        sender_pod: None,
        timestamp: Some(chrono::Utc::now().timestamp_millis() as u64),
    };

    publish_message(app_state, &message).await
}

pub async fn start_redis_subscriber(app_state: AppState) {
    tokio::spawn(async move {
        loop {
            let result = subscribe_and_handle(&app_state).await;
            if result.is_err() {
                eprintln!("Redis subscriber error, retrying in 5 seconds...");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    });
}

async fn subscribe_and_handle(
    app_state: &AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut pubsub = client.get_async_pubsub().await?;
    pubsub.subscribe("socket:messages").await?;

    let mut msg_stream = pubsub.on_message();

    loop {
        if let Some(msg) = msg_stream.next().await {
            let payload: String = msg.get_payload()?;

            if let Ok(redis_message) = serde_json::from_str::<RedisMessage>(&payload) {
                handle_redis_message(app_state, redis_message).await;
            }
        }
    }
}

async fn handle_redis_message(app_state: &AppState, message: RedisMessage) {
    let target_email = message.target_email.clone();
    let target_device = message.target_device.clone();

    // Check if this message is for a user on this pod
    let email_device_map = app_state.email_device_to_socket.read().await;

    if let Some(device_map) = email_device_map.get(&target_email) {
        if let Some(socket_id) = device_map.get(&target_device) {
            // User is on this pod, forward the message
            let socket_connections = app_state.socket_id_to_connection.read().await;

            if let Some(tx) = socket_connections.get(socket_id) {
                let msg_text = serde_json::to_string(&message.socket_message).unwrap_or_default();
                let _ = tx.send(Message::Text(msg_text.into())).await;

                // TODO: Send delivery confirmation back to originating pod
                // This would require tracking pending messages and having a confirmation channel
            }
        }
    }

    // Handle broadcast messages (user_joined, user_left)
    if target_email == "*" && target_device == "*" {
        // These are handled by the client receiving the message
        // No additional server-side action needed
    }
}
