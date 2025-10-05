mod config;
mod handlers;
mod middleware;
mod models;

use axum::{Router, middleware as axum_middleware, routing::get};

use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use config::Config;
use handlers::files;
use handlers::health;
use middleware::logging::logging_middleware;

#[tokio::main]
async fn main() {
    // Load configuration from environment variables
    let config = Config::from_env().expect("Failed to load configuration");

    // Initialize logging with the configured log level
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| config.log_level.clone().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("📁 Root directory: {}", config.root_dir);
    tracing::info!("🔧 Log level: {}", config.log_level);

    // Wrap config in Arc for sharing across threads
    let shared_config = Arc::new(config.clone());

    // Create CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build API routes
    let api_routes = Router::new()
        .route("/files", get(files::get_files))
        .route("/videos", get(files::get_videos))
        .with_state(shared_config.clone());

    // Build main app with all routes and middleware
    let app = Router::new()
        .route("/health", get(health::health_handler))
        .nest("/api/v1", api_routes)
        .layer(axum_middleware::from_fn(logging_middleware))
        .layer(cors);

    // Define the server address
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));

    tracing::info!("🚀 Server starting on http://{}", addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app).await.expect("Server error");
}
