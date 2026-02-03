use axum::http::StatusCode;
use axum::{Json, Router, extract::State, response::IntoResponse, routing::post};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::db::models::login_token::LoginToken;
use crate::db::models::user::User;

#[derive(Deserialize, Debug)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
}

pub fn login(state: AppState) -> Router {
    Router::new()
        .route("/", post(login_handler))
        .with_state(state)
}

async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let user_id: Uuid = match User::validate_login(payload.email, payload.password, state.clone())
        .await
    {
        Ok(id) => Uuid::parse_str(id.as_str()).unwrap(),
        Err(_) => return (StatusCode::UNAUTHORIZED, "Invalid email or password").into_response(),
    };

    match LoginToken::create(user_id, state).await {
        Ok(token) => Json(LoginResponse {
            token: token.to_string(),
        })
        .into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create token").into_response(),
    }
}
