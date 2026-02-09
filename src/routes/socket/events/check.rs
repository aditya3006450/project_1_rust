use serde_json::Value;
use std::str::FromStr;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    db::models::user_connection::UserConnection,
    routes::socket::{
        redis_manager::get_user_devices,
        types::{DeviceInfo, UserDevicesResponse},
    },
};

pub async fn check_users(
    from_email: String,
    app_state: AppState,
) -> Vec<UserDevicesResponse> {
    // Get the user ID from email (assuming email is UUID)
    let Ok(user_uuid) = Uuid::from_str(from_email.as_str()) else {
        return Vec::new();
    };

    // Get connected users from database
    let connected_users_result = UserConnection::connected_to(user_uuid, app_state.clone()).await;

    let connected_users = match connected_users_result {
        Ok(users) => users,
        Err(_) => return Vec::new(),
    };

    let mut responses = Vec::new();

    // For each connected user, get their devices from Redis
    for user in connected_users {
        let email = user.from_email;

        match get_user_devices(&app_state, &email).await {
            Ok(devices) => {
                if !devices.is_empty() {
                    responses.push(UserDevicesResponse {
                        email,
                        devices,
                    });
                }
            }
            Err(_) => {
                // If Redis fails, try local fallback
                let user_index = app_state.user_index.read().await;
                if let Some(device_map) = user_index.get(&email) {
                    let local_devices: Vec<DeviceInfo> = device_map
                        .iter()
                        .filter_map(|(_device_id, value)| {
                            serde_json::from_value::<DeviceInfo>(value.clone()).ok()
                        })
                        .collect();

                    if !local_devices.is_empty() {
                        responses.push(UserDevicesResponse {
                            email,
                            devices: local_devices,
                        });
                    }
                }
            }
        }
    }

    responses
}

pub async fn check_users_response(
    from_email: String,
    app_state: AppState,
) -> Value {
    let users = check_users(from_email, app_state).await;
    serde_json::to_value(users).unwrap_or_default()
}
