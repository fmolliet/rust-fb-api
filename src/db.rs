use sqlx::PgPool;

pub async fn create_pool(database_url: &str) -> PgPool {
    PgPool::connect(database_url).await.expect("Failed to connect to database")
}

pub async fn health_check(pool: &PgPool) {
    sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await
        .expect("Database health check failed");
}
