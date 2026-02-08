use axum::extract::ws::{Message, WebSocket};
use futures_util::stream::SplitSink;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use bb8_redis::RedisConnectionManager;
use sqlx::PgPool;

use crate::utils::{mail_service::mailer::Mailer, tera_service::tera_renderer::TeraRenderer};
type RedisPool = bb8::Pool<RedisConnectionManager>;

pub type Tx = SplitSink<WebSocket, Message>;

#[derive(Clone)]
pub struct AppState {
    pub pg_pool: PgPool,
    pub redis_pool: RedisPool,
    pub mailer: Arc<Mailer>,
    pub tera_renderer: Arc<TeraRenderer>,
    pub user_index: HashMap<String, HashMap<String, Value>>,
    pub socket_connections: Arc<RwLock<HashMap<String, Tx>>>,
}
