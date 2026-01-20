use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("DATABASE_URL must be set")]
    MissingDatabaseUrl,
    #[error(transparent)]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

pub async fn connect_from_env_and_migrate() -> Result<PgPool, DbError> {
    let database_url = std::env::var("DATABASE_URL").map_err(|_| DbError::MissingDatabaseUrl)?;
    connect_and_migrate(&database_url).await
}

pub async fn connect_and_migrate(database_url: &str) -> Result<PgPool, DbError> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(10))
        .connect(database_url)
        .await?;

    migrate(&pool).await?;
    Ok(pool)
}

pub async fn migrate(pool: &PgPool) -> Result<(), DbError> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}
