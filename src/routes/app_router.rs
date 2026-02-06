use axum::{Router, routing::get};

use crate::{
    app_state::AppState,
    routes::{auth::auth_router::auth_router, user_connection::user_connection::user_connection},
    utils::auth_middleware::auth_middleware,
};

pub fn app_router(state: AppState) -> Router {
    Router::new().merge(home(state))
}

fn home(state: AppState) -> Router {
    Router::new()
        .route("/", get("pong"))
        .nest("/auth", auth_router(state.clone()))
        .nest(
            "/user-connection",
            user_connection(state.clone()).layer(axum::middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            )),
        )
}
