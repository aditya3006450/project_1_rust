use redis::AsyncCommands;
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::{app_state::AppState, utils::hash_service::bcrypt::verify_password};

#[derive(Debug, FromRow)]
pub struct User {
    id: Uuid,
    email: String,
    password_hash: String,
}

impl User {
    pub async fn cache_user(
        email: String,
        user_id: String,
        password_hash: String,
        app_state: AppState,
    ) -> Result<(), redis::RedisError> {
        let mut redis_connection = match app_state.redis_pool.get().await {
            Ok(conn) => conn,
            Err(_) => {
                eprintln!("Redis connection failed");
                return Ok(());
            }
        };

        let key = format!("auth:{}", email);

        let _: Result<(), redis::RedisError> = redis_connection
            .hset_multiple(
                &key,
                &[
                    ("user_id", user_id.to_string()),
                    ("password_hash", password_hash.clone()),
                ],
            )
            .await;

        redis_connection.expire(&key, 3600).await
    }
    pub async fn create(
        email: String,
        password_hash: String,
        app_state: AppState,
    ) -> Result<(), sqlx::Error> {
        let mut tx = app_state.pg_pool.begin().await?;
        let user_id = sqlx::query_scalar!(
            "INSERT INTO USERS (email,password_hash)
            values ($1,$2)
            RETURNING id",
            email.clone(),
            password_hash.clone()
        )
        .fetch_one(&mut *tx)
        .await?;
        User::cache_user(email, user_id.to_string(), password_hash, app_state)
            .await
            .unwrap();

        tx.commit().await?;
        Ok(())
    }

    pub async fn validate_login(
        email: String,
        password: String,
        app_state: AppState,
    ) -> Result<String, sqlx::Error> {
        let mut redis_connection = app_state
            .redis_pool
            .get()
            .await
            .map_err(|_| sqlx::Error::PoolClosed)?; // safer than unwrap

        let key = format!("auth:{}", email);

        if let Ok(Some(user_id)) = redis_connection
            .hget::<_, _, Option<String>>(&key, "user_id")
            .await
        {
            let password_hash: String = redis_connection
                .hget(&key, "password_hash")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

            if verify_password(password.clone(), password_hash) {
                return Ok(user_id);
            } else {
                return Err(sqlx::Error::RowNotFound);
            }
        }

        let mut tx = app_state.pg_pool.begin().await?;
        let row = sqlx::query!(
            "SELECT id, password_hash FROM users WHERE email = $1",
            email
        )
        .fetch_one(&mut *tx)
        .await?;

        if !verify_password(password.clone(), row.password_hash.clone()) {
            return Err(sqlx::Error::RowNotFound);
        }

        User::cache_user(
            email,
            row.id.to_string(),
            row.password_hash.clone(),
            app_state.clone(),
        )
        .await
        .map_err(|_| sqlx::Error::Protocol("Failed to cache user".to_string()))?;

        tx.commit().await?;

        Ok(row.id.to_string())
    }
}
