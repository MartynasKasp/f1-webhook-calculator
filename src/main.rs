mod error;
mod db;
mod handlers;
mod models;

use axum::{Router, routing::post};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info,sqlx=warn"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = db::create_pool(&database_url).await.expect("Failed to create pool");

    // Run migrations in dev (in prod use sqlx migrate run)
    // sqlx::migrate!().run(&pool).await.unwrap();

    let app = Router::new()
        .route("/webhook/calculate-permutation", post(handlers::calculate_permutation_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8090").await.unwrap();
    tracing::info!("Listening on http://0.0.0.0:8090");
    axum::serve(listener, app).await.unwrap();
}