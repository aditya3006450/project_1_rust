use chrono::{DateTime, Utc};
use redis::{AsyncCommands, RedisError};
use uuid::Uuid;

use crate::app_state::AppState;

pub struct LoginToken {
    id: Uuid,
    user_id: Uuid,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

impl LoginToken {
    async fn cache_token(
        token_id: Uuid,
        user_id: Uuid,
        app_state: AppState,
    ) -> Result<(), RedisError> {
        let key = format!("login_token:{}", token_id.clone());
        let mut redis_connetion = app_state.redis_pool.get().await.unwrap();
        let res = redis_connetion
            .set_ex(key, user_id.to_string(), 24 * 60 * 60)
            .await;
        res
    }

    pub async fn create(user_id: Uuid, app_state: AppState) -> Result<Uuid, sqlx::Error> {
        let mut tx = app_state.pg_pool.begin().await?;
        let rec = sqlx::query!(
            r#"
            INSERT INTO user_tokens (user_id, expires_at)
            VALUES ($1, NOW() + INTERVAL '16 hours')
            RETURNING id
            "#,
            user_id
        )
        .fetch_one(&mut *tx)
        .await;
        let token_id = rec.unwrap().id;
        let _ = LoginToken::cache_token(token_id, user_id, app_state);
        let _ = tx.commit().await;

        Ok(token_id)
    }
    pub async fn get_user_id(token_id: Uuid, app_state: AppState) -> Result<String, sqlx::Error> {
        let key = format!("login_token:{}", token_id.clone());
        let mut redis_connection = app_state.redis_pool.get().await.unwrap();
        let cached_user_id: Option<String> = redis_connection.get(&key).await.unwrap();
        if let Some(id) = cached_user_id {
            Ok(id)
        } else {
            let mut tx = app_state.clone().pg_pool.begin().await?;
            let row = sqlx::query!("SELECT user_id FROM user_tokens WHERE id = $1", token_id)
                .fetch_one(&mut *tx)
                .await?;
            let _ = LoginToken::cache_token(token_id, row.user_id.clone(), app_state.clone())
                .await
                .unwrap();
            let _ = tx.commit().await;
            Ok(row.user_id.to_string())
        }
    }
}
