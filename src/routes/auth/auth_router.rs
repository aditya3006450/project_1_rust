use axum::Router;

use crate::{
    app_state::AppState,
    routes::auth::{login::login, setup_password::setup_password, signup::signup},
};

pub fn auth_router(state: AppState) -> Router {
    Router::new()
        .nest("/login", login(state.clone()))
        .nest("/signup", signup(state.clone()))
        .nest("/setup-password", setup_password(state.clone()))
}
