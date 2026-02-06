use std::str::FromStr;

use axum::{
    Extension, Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{app_state::AppState, db::models::user_connection::UserConnection};

#[derive(Deserialize, Debug)]
pub struct SentRequestBody {
    to_email: String,
}

pub fn user_connection(state: AppState) -> Router {
    Router::new()
        .route(
            "/send_request",
            post(send_request).with_state(state.clone()),
        )
        .route(
            "/accept_request",
            post(accept_request).with_state(state.clone()),
        )
        .route(
            "/sent_requests",
            get(sent_requests).with_state(state.clone()),
        )
        .route(
            "/recieved_requests",
            get(recieved_requests).with_state(state.clone()),
        )
        .route("/connected_to", get(connected_to).with_state(state.clone()))
        .route(
            "/connected_from",
            get(connected_from).with_state(state.clone()),
        )
}

pub async fn send_request(
    Extension(user_id): Extension<String>,
    State(state): State<AppState>,
    Json(payload): Json<SentRequestBody>,
) -> impl IntoResponse {
    let u_id = Uuid::from_str(user_id.as_str()).unwrap();
    match UserConnection::add_request(u_id, payload.to_email, state).await {
        Ok(_) => StatusCode::CREATED,
        Err(e) => {
            println!("{e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn accept_request(
    Extension(user_id): Extension<String>,
    State(state): State<AppState>,
    Json(payload): Json<SentRequestBody>,
) -> impl IntoResponse {
    let u_id = Uuid::from_str(user_id.as_str()).unwrap();
    match UserConnection::add_connection(u_id, payload.to_email, state).await {
        Ok(_) => StatusCode::CREATED,
        Err(e) => {
            println!("{e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn sent_requests(
    Extension(user_id): Extension<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let u_id = Uuid::from_str(user_id.as_str()).unwrap();
    if let Ok(user_connections) = UserConnection::get_sent_requests(u_id, state).await {
        (
            StatusCode::OK,
            Json(serde_json::json!({"res": user_connections})),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"res": []})),
        )
    }
}

pub async fn recieved_requests(
    Extension(user_id): Extension<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let u_id = Uuid::from_str(user_id.as_str()).unwrap();
    if let Ok(user_connections) = UserConnection::get_recieved_requests(u_id, state).await {
        (
            StatusCode::OK,
            Json(serde_json::json!({"res": user_connections})),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"res": []})),
        )
    }
}

pub async fn connected_from(
    Extension(user_id): Extension<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let u_id = Uuid::from_str(user_id.as_str()).unwrap();
    if let Ok(user_connections) = UserConnection::connected_from(u_id, state).await {
        (
            StatusCode::OK,
            Json(serde_json::json!({"res": user_connections})),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"res": []})),
        )
    }
}

pub async fn connected_to(
    Extension(user_id): Extension<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let u_id = Uuid::from_str(user_id.as_str()).unwrap();
    if let Ok(user_connections) = UserConnection::connected_to(u_id, state).await {
        (
            StatusCode::OK,
            Json(serde_json::json!({"res": user_connections})),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"res": []})),
        )
    }
}
