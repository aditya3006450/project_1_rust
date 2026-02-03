use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    utils::{
        hash_service::hash_generator::generate_hash, mail_service::mail_data::MailData,
        resolve_base_url::resolve_base_url,
    },
};

#[derive(Deserialize, Debug)]
struct SignupRequest {
    email: String,
}

#[derive(Serialize)]
struct SignupResponse {
    email: String,
}

pub fn signup(state: AppState) -> Router {
    Router::new()
        .route("/", post(signup_handler))
        .with_state(state)
}

async fn signup_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SignupRequest>,
) -> impl IntoResponse {
    // todo: make a time based email sending limiter
    let token = generate_hash();
    let signup_url = format!(
        "{}/auth/setup-password?token={}",
        resolve_base_url(&headers),
        token.clone()
    );

    let conn = state
        .redis_pool
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);

    let _: () = conn
        .unwrap()
        .set_ex(token, payload.email.clone(), 600)
        .await
        .unwrap();

    let mail = MailData::with_template(
        String::from(payload.email.clone()),
        "Setup Password".into(),
        "mails/signup.html".into(),
        serde_json::json!({ "signup_url":signup_url }),
    );

    tokio::spawn(async move {
        let res = state.mailer.send(&state.tera_renderer, mail.clone()).await;
        res.expect(&format!("{:?} email could not be sent", mail.clone()))
        // todo: save it in paper trails or make it safe in some way
    });

    (
        StatusCode::CREATED,
        Json(SignupResponse {
            email: payload.email,
        }),
    )
}
