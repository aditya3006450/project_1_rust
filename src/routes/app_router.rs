use axum::{Router, routing::get};

use crate::{app_state::AppState, routes::auth::auth_router::auth_router};

pub fn app_router(state: AppState) -> Router {
    Router::new().merge(home(state))
}

fn home(state: AppState) -> Router {
    Router::new()
        .route("/", get("hello world"))
        .nest("/auth", auth_router(state))
}
