use axum::{
    Router,
    routing::{get, post},
};

use crate::app_state::AppState;

fn user_connection(state: AppState) -> Router {
    Router::new()
        .nest("/sent_requests", post("hello"))
        .nest("/accept_request", post("which one"))
        .nest("/sent_requests", get("sent_requests"))
        .nest("/recieved_requests", get("recieved_requests"))
}
