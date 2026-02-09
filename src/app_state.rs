use axum::extract::ws::Message;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, RwLock};

use bb8_redis::RedisConnectionManager;
use sqlx::PgPool;

use crate::utils::{mail_service::mailer::Mailer, tera_service::tera_renderer::TeraRenderer};
type RedisPool = bb8::Pool<RedisConnectionManager>;

pub type Tx = mpsc::Sender<Message>;

#[derive(Clone)]
pub struct AppState {
    pub pg_pool: PgPool,
    pub redis_pool: RedisPool,
    pub mailer: Arc<Mailer>,
    pub tera_renderer: Arc<TeraRenderer>,
    // email -> { device_id -> DeviceInfo }
    pub user_index: Arc<RwLock<HashMap<String, HashMap<String, Value>>>>,
    // email_device_key -> tx (for backward compatibility during migration)
    pub socket_connections: Arc<RwLock<HashMap<String, Tx>>>,
    // socket_id -> tx (new mapping)
    pub socket_id_to_connection: Arc<RwLock<HashMap<String, Tx>>>,
    // email -> { device_id -> socket_id }
    pub email_device_to_socket: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
}

impl AppState {
    pub fn get_redis_presence_key(email: &str, device_id: &str) -> String {
        format!("socket:presence:{}:{}", email, device_id)
    }

    pub fn get_redis_user_devices_key(email: &str) -> String {
        format!("socket:user_devices:{}", email)
    }
}
