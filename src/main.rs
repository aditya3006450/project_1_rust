use std::sync::Arc;

use dotenv::dotenv;

use crate::utils::mail_service::mailer::Mailer;
use crate::utils::tera_service::tera_renderer::TeraRenderer;
use crate::{app_state::AppState, db::connect_db::connect_db};
mod app_state;
mod db;
mod routes;
mod utils;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let (pg_pool, redis_pool) = connect_db().await.expect("Failed to connect to databases");
    let tera_renderer = Arc::new(TeraRenderer::new());
    let mailer = Arc::new(Mailer::new());
    let app_state = AppState {
        redis_pool,
        pg_pool,
        tera_renderer,
        mailer,
    };
    let router = routes::app_router::app_router(app_state);
    let listner = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listner, router).await.unwrap()
}
