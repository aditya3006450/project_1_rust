use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::app_state::AppState;

#[derive(Debug, FromRow)]
pub struct UserConnection {
    id: Uuid,
    from_id: Uuid,
    to_id: Uuid,
    is_accepted: bool,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct UserConnectionView {
    pub from_email: String,
    pub to_email: String,
    pub is_accepted: bool,
}

impl UserConnection {
    // called after accepting the request
    pub async fn add_connection(
        from_id: Uuid,
        to_email: String,
        app_state: AppState,
    ) -> Result<(), sqlx::Error> {
        let mut tx = app_state.pg_pool.begin().await?;
        sqlx::query!(
            "UPDATE user_connection uc SET is_accepted = true FROM users u where u.email = $2 and uc.from_id = $1 and uc.to_id = u.id",
            from_id,
            to_email
        )
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }

    // for sending the request
    pub async fn add_request(
        from_id: Uuid,
        to_email: String,
        app_state: AppState,
    ) -> Result<(), sqlx::Error> {
        let mut tx = app_state.pg_pool.begin().await?;
        println!("from_id: {from_id} to_id: {to_email}");
        sqlx::query!(
            "INSERT into user_connection (from_id, to_id, is_accepted) select $1,id,false from users where email = $2",
            from_id,
            to_email
        )
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn get_sent_requests(
        from_id: Uuid,
        app_state: AppState,
    ) -> Result<Vec<UserConnectionView>, sqlx::Error> {
        let rows = sqlx::query_as!(
            UserConnectionView,
            r#"
            SELECT 
                u1.email AS from_email, 
                u2.email AS to_email, 
                uc.is_accepted
            FROM user_connection uc
            JOIN users u1 ON uc.from_id = u1.id
            JOIN users u2 ON uc.to_id = u2.id
            WHERE uc.from_id = $1 AND uc.is_accepted = false
        "#,
            from_id
        )
        .fetch_all(&app_state.pg_pool)
        .await?;

        Ok(rows)
    }

    pub async fn get_recieved_requests(
        to_id: Uuid,
        app_state: AppState,
    ) -> Result<Vec<UserConnectionView>, sqlx::Error> {
        let rows = sqlx::query_as!(
            UserConnectionView,
            r#"
            SELECT 
                u1.email AS from_email, 
                u2.email AS to_email, 
                uc.is_accepted
            FROM user_connection uc
            JOIN users u1 ON uc.from_id = u1.id
            JOIN users u2 ON uc.to_id = u2.id
            WHERE uc.to_id = $1 AND uc.is_accepted = false
        "#,
            to_id
        )
        .fetch_all(&app_state.pg_pool)
        .await?;

        Ok(rows)
    }

    pub async fn connected_to(
        to_id: Uuid,
        app_state: AppState,
    ) -> Result<Vec<UserConnectionView>, sqlx::Error> {
        let rows = sqlx::query_as!(
            UserConnectionView,
            r#"
            SELECT 
                u1.email AS from_email, 
                u2.email AS to_email, 
                uc.is_accepted
            FROM user_connection uc
            JOIN users u1 ON uc.from_id = u1.id
            JOIN users u2 ON uc.to_id = u2.id
            WHERE uc.to_id = $1 AND uc.is_accepted = true 
        "#,
            to_id
        )
        .fetch_all(&app_state.pg_pool)
        .await?;

        Ok(rows)
    }

    pub async fn connected_from(
        from_id: Uuid,
        app_state: AppState,
    ) -> Result<Vec<UserConnectionView>, sqlx::Error> {
        let rows = sqlx::query_as!(
            UserConnectionView,
            r#"
            SELECT 
                u1.email AS from_email, 
                u2.email AS to_email, 
                uc.is_accepted
            FROM user_connection uc
            JOIN users u1 ON uc.from_id = u1.id
            JOIN users u2 ON uc.to_id = u2.id
            WHERE uc.from_id = $1 AND uc.is_accepted = true
        "#,
            from_id
        )
        .fetch_all(&app_state.pg_pool)
        .await?;

        Ok(rows)
    }
}
