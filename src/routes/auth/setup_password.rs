use std::collections::HashMap;

use axum::{
    Form, Router,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::Html,
    routing::{get, post},
};
use redis::AsyncCommands;
use serde::Deserialize;
use serde_json::json;

use crate::{
    app_state::AppState,
    db::models::user::User,
    utils::{hash_service::bcrypt::hash_password, resolve_base_url::resolve_base_url},
};
#[derive(Deserialize, Debug)]
pub struct UserPassword {
    password: String,
}

pub fn setup_password(state: AppState) -> Router {
    Router::new()
        .route("/", get(password_setup).with_state(state.clone()))
        .route("/", post(password_setup_confirmation).with_state(state))
}

async fn password_setup_confirmation(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    Form(payload): Form<UserPassword>,
) -> Html<String> {
    let token = params.get("token").unwrap();
    let password = payload.password;
    let connection = state
        .redis_pool
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    let email_value: Result<String, StatusCode> = connection
        .unwrap()
        .get(token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);

    match email_value {
        Ok(email) => {
            let password_hash = hash_password(password.to_string());
            match User::create(email, password_hash, state.clone()).await {
                Ok(_) => {
                    let html = state
                        .tera_renderer
                        .render("pages/password-setup-success.html", json!({}))
                        .unwrap();
                    Html(html)
                }
                Err(_) => {
                    let html = state
                        .tera_renderer
                        .render("pages/something-went-wrong.html", json!({}))
                        .unwrap();
                    Html(html)
                }
            }
        }
        Err(_) => Html("Something went wrong".to_string()),
    }
}

async fn password_setup(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Html<String> {
    let token = params.get("token").unwrap();
    let connection = state
        .redis_pool
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    let email_value: Result<String, StatusCode> = connection
        .unwrap()
        .get(token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);

    match email_value {
        Ok(email) => {
            let password_setup_url = format!(
                "{}/auth/setup-password?token={}",
                resolve_base_url(&headers),
                token
            );
            let context = json!({ "password_setup_url": password_setup_url,"email": email});
            let html = state
                .tera_renderer
                .render("pages/setup-password.html", context)
                .unwrap();
            Html(html)
        }
        Err(_) => Html("Link expired or invalid".to_string()),
    }
}
