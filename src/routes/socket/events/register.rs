use serde_json::json;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    db::models::{login_token::LoginToken, user::User},
    routes::socket::{
        redis_manager::{broadcast_user_joined, store_device_presence},
        types::{DeviceInfo, SocketMessage},
    },
};

pub async fn register_user(
    message: SocketMessage,
    device_id: String,
    socket_id: &str,
    app_state: AppState,
) -> Result<(), String> {
    // Validate the token
    let token_id = match Uuid::parse_str(&message.from_token) {
        Ok(id) => id,
        Err(_) => return Err("Invalid token format".to_string()),
    };

    let user_id = match LoginToken::get_user_id(token_id, app_state.clone()).await {
        Ok(id) => id,
        Err(_) => return Err("Invalid or expired token".to_string()),
    };

    // Verify the email matches the token
    if User::get_user_email(user_id, app_state.clone())
        .await
        .unwrap()
        != message.from_email
    {
        return Err("Email does not match token".to_string());
    }

    // Check if this device is already registered (same device reconnecting)
    // Clean up old connection if exists
    {
        let email_device_map = app_state.email_device_to_socket.read().await;
        if let Some(device_map) = email_device_map.get(&message.from_email) {
            if let Some(old_socket_id) = device_map.get(&message.from_device) {
                // Device already exists, clean up old socket
                let old_socket_id = old_socket_id.clone();
                drop(email_device_map);

                // Remove old socket connection
                app_state
                    .socket_id_to_connection
                    .write()
                    .await
                    .remove(&old_socket_id);

                // Remove from old socket_connections map
                let old_key = format!("{}{}", message.from_email, message.from_device);
                app_state.socket_connections.write().await.remove(&old_key);

                println!(
                    "Cleaned up old socket {} for device {}",
                    old_socket_id, message.from_device
                );
            }
        }
    }

    // Create device info from payload
    let device_name = message
        .payload
        .get("device_name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let device_type = message
        .payload
        .get("device_type")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let device_info = DeviceInfo {
        socket_id: socket_id.to_string(),
        device_name,
        device_type,
        device_id,
    };

    // Store in Redis for cross-pod visibility
    if let Err(e) = store_device_presence(
        &app_state,
        &message.from_email,
        &message.from_device,
        &device_info,
    )
    .await
    {
        eprintln!("Failed to store device presence in Redis: {}", e);
        // Continue in local-only mode
    } else {
        // Broadcast to other pods that user joined
        if let Err(e) =
            broadcast_user_joined(&app_state, &message.from_email, &message.from_device).await
        {
            eprintln!("Failed to broadcast user joined: {}", e);
        }
    }

    // Also store locally for fast access
    let mut user_index = app_state.user_index.write().await;
    if let Some(devices) = user_index.get_mut(&message.from_email) {
        devices.insert(message.from_device.clone(), json!(&device_info));
    } else {
        let mut map = std::collections::HashMap::new();
        map.insert(message.from_device.clone(), json!(&device_info));
        user_index.insert(message.from_email.clone(), map);
    }

    Ok(())
}
