use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;

use uuid::Uuid;

use crate::db::models::user_connection::UserConnection;
use crate::{
    app_state::AppState, db::models::user_connection::UserConnectionView,
    routes::socket::types::SocketMessage,
};

pub async fn ping_users(
    message: SocketMessage,
    app_state: AppState,
) -> Vec<HashMap<std::string::String, Value>> {
    let user_id = message.from_email;
    let connected_users: Vec<UserConnectionView> = UserConnection::connected_to(
        Uuid::from_str(user_id.as_str()).ok().unwrap(),
        app_state.clone(),
    )
    .await
    .unwrap();

    let user_emails: Vec<String> = connected_users
        .iter()
        .map(|user| user.to_email.to_string())
        .collect();

    let mut res: Vec<HashMap<String, Value>> = Vec::new();
    let user_index = app_state.user_index.read().await;
    for email in user_emails {
        if let Some(value) = user_index.get(&email) {
            res.push(value.clone());
        }
    }
    // todo: when going through multi pod system
    // think to combine all the lists together
    // return the device ids combined

    res
}
