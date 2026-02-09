use axum::extract::ws::WebSocket;
use serde_json::Value;
use std::collections::HashMap;

use crate::{app_state::AppState, routes::socket::types::SocketMessage};

pub async fn register_user(message: SocketMessage, socket: &mut WebSocket, app_state: AppState) {
    let mut user_index = app_state.user_index.write().await;
    if let Some(devices) = user_index.get_mut(&message.from_email) {
        devices.insert(message.from_device.clone(), message.payload);
        let key = format!("{}{}", message.from_email, message.from_device);
        app_state
            .socket_connections
            .write()
            .await
            .insert(key, socket);
    } else {
        let map: HashMap<String, Value> = HashMap::from([(message.from_device, message.payload)]);
        user_index.insert(message.from_email, map);
        app_state
            .socket_connections
            .write()
            .await
            .insert(key, socket);
    }
}
