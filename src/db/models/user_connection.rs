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

#[derive(Debug, FromRow)]
pub struct UserConnectionView {
    pub from_id: Uuid,
    pub to_id: Uuid,
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
            "INSERT into user_connection (from_id, to_id, is_accepted) select $1,id,true from users where email = $2",
            from_id,
            to_email
        )
        .fetch_one(&mut *tx)
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
        sqlx::query!(
            "INSERT into user_connection (from_id, to_id, is_accepted) select $1,id,true from users where email = $2",
            from_id,
            to_email
        )
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }

    // checking requests
    pub async fn get_sent_requests(
        from_id: Uuid,
        app_state: AppState,
    ) -> Result<Vec<UserConnectionView>, sqlx::Error> {
        let rows = sqlx::query_as!(
            UserConnectionView,
            r#"
                SELECT from_id, to_id, is_accepted
                FROM user_connection
                WHERE from_id = $1 AND is_accepted = false
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
                SELECT from_id, to_id, is_accepted
                FROM user_connection
                WHERE to_id = $1 AND is_accepted = false
            "#,
            to_id
        )
        .fetch_all(&app_state.pg_pool)
        .await?;
        Ok(rows)
    }
}
