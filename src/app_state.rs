use std::sync::Arc;

use bb8_redis::RedisConnectionManager;
use sqlx::PgPool;

use crate::utils::{mail_service::mailer::Mailer, tera_service::tera_renderer::TeraRenderer};
type RedisPool = bb8::Pool<RedisConnectionManager>;

#[derive(Clone)]
pub struct AppState {
    pub pg_pool: PgPool,
    pub redis_pool: RedisPool,
    pub mailer: Arc<Mailer>,
    pub tera_renderer: Arc<TeraRenderer>,
}
