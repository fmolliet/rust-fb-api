use fbrustapi::{config, db, routes, middleware};
use axum::{routing::get, Router, middleware::from_fn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "fbrustapi=debug,sqlx_core=warn".into()))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let config = config::Config::from_env();
    let pool = db::create_pool(&config.database_url).await;

    tracing::info!("Connected to database");

    db::health_check(&pool).await;
    tracing::info!("Health check passed");

    let state = db::AppState { pool };

    let app = Router::new()
        .route("/health", get(health))
        .nest("/registration", routes::register::register_routes())
        .nest("/birthday", routes::birthday::birthday_routes())
        .nest("/meeting", routes::meeting::meeting_routes())
        .nest("/announce", routes::announce::announce_routes())
        .layer(from_fn(middleware::correlation_id_middleware))
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8081".to_string());
    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> &'static str {
    "ok"
}
