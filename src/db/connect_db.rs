use bb8_redis::RedisConnectionManager;
use sqlx::{PgPool, postgres::PgPoolOptions};

type RedisPool = bb8::Pool<RedisConnectionManager>;
pub async fn connect_db() -> Result<(PgPool, RedisPool), Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL not set");
    let manager = RedisConnectionManager::new(redis_url)?;
    let redis_pool = bb8::Pool::builder().max_size(15).build(manager).await?;

    let pg_pool = PgPoolOptions::new().connect(&database_url).await?;
    Ok((pg_pool, redis_pool))
}
