use fbrustapi::{config, db};
use axum::routing::get;
use axum::Router;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "fbrustapi=debug,sqlx_core=warn".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::Config::from_env();
    let pool = db::create_pool(&config.database_url).await;

    tracing::info!("Connected to database");

    db::health_check(&pool).await;
    tracing::info!("Health check passed");

    let app = Router::new()
        .route("/health", get(health));

    let addr = "0.0.0.0:8080";
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> &'static str {
    "ok"
}
