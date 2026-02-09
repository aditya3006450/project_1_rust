use crate::{
    app_state::AppState,
    routes::socket::redis_manager::{broadcast_user_left, remove_device_presence},
};

pub async fn disconnect_user(
    email: String,
    device: String,
    socket_id: String,
    state: AppState,
) {
    // Remove from Redis
    if let Err(e) = remove_device_presence(&state, &email, &device).await {
        eprintln!("Failed to remove device presence from Redis: {}", e);
    } else {
        // Broadcast to other pods that user left
        if let Err(e) = broadcast_user_left(&state, &email, &device).await {
            eprintln!("Failed to broadcast user left: {}", e);
        }
    }

    // Remove from local mappings
    let key = format!("{}{}", email, device);
    state.socket_connections.write().await.remove(&key);

    // Remove from socket_id mapping
    state.socket_id_to_connection.write().await.remove(&socket_id);

    // Remove from email_device mapping
    let mut email_device_map = state.email_device_to_socket.write().await;
    if let Some(device_map) = email_device_map.get_mut(&email) {
        device_map.remove(&device);
        if device_map.is_empty() {
            email_device_map.remove(&email);
        }
    }

    // Remove from user_index
    let mut user_index = state.user_index.write().await;
    if let Some(user_devices) = user_index.get_mut(&email) {
        user_devices.remove(&device);
        if user_devices.is_empty() {
            user_index.remove(&email);
        }
    }
}
