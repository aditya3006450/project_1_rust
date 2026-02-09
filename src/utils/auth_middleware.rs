use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::db::models::login_token::LoginToken;

pub async fn auth_middleware(
    State(app_state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());
    if let Some(token) = auth_header {
        println!("{token}");
        if let Ok(user_id) =
            LoginToken::get_user_id(Uuid::parse_str(token).unwrap(), app_state).await
        {
            req.extensions_mut().insert(user_id);
            return Ok(next.run(req).await);
        }
    }
    Err(StatusCode::UNAUTHORIZED)
}
