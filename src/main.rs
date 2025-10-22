mod config;
mod handlers;
mod middleware;
mod models;

use axum::{
    Router,
    http::StatusCode,
    middleware as axum_middleware,
    response::IntoResponse,
    routing::{get, post},
};

use axum::body::Body;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
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
        .route("/search", get(files::search))
        .route("/file/{*wildcard}", get(files::serve_file))
        .route("/stream/{*wildcard}", get(files::stream_file))
        .route("/thumbnail/{*wildcard}", get(files::get_thumbnail))
        .route("/createfolder", post(files::create_folder))
        .route("/uploadfile", post(files::upload_file))
        .route(
            "/syncfusion/fileoperations",
            post(handlers::syncfusion::file_operations),
        )
        .with_state(shared_config.clone());

    // Check if webdeploy directory exists
    let serve_static = std::path::Path::new("./webdeploy").exists();

    if serve_static {
        tracing::info!("✅ Serving Blazor WebAssembly files from ./webdeploy");
    } else {
        tracing::warn!("⚠️  webdeploy directory not found, Blazor UI will not be available");
    }

    // Build main app with all routes and middleware
    let app = if serve_static {
        // Serve static files and handle SPA routing
        Router::new()
            .route("/health", get(health::health_handler))
            .nest("/api/v1", api_routes)
            .fallback_service(
                ServeDir::new("webdeploy").not_found_service(tower::service_fn(spa_handler)),
            )
            .layer(
                ServiceBuilder::new()
                    .layer(axum_middleware::from_fn(logging_middleware))
                    .layer(cors),
            )
    } else {
        // No static files, just API
        Router::new()
            .route("/health", get(health::health_handler))
            .nest("/api/v1", api_routes)
            .layer(axum_middleware::from_fn(logging_middleware))
            .layer(cors)
    };

    // Define the server address
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));

    tracing::info!("🚀 Server starting on http://{}", addr);

    if serve_static {
        tracing::info!(
            "🌐 Access the web interface at: http://localhost:{}",
            config.port
        );
    }
    tracing::info!(
        "📡 API available at: http://localhost:{}/api/v1",
        config.port
    );

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app).await.expect("Server error");
}

// Handler for SPA fallback - serves index.html for client-side routing
async fn spa_handler(
    _req: axum::http::Request<Body>,
) -> Result<axum::response::Response, Infallible> {
    match tokio::fs::read_to_string("webdeploy/index.html").await {
        Ok(contents) => {
            let body = Body::from(contents);
            let response = axum::response::Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "text/html; charset=utf-8")
                .body(body)
                .unwrap(); // safe: valid status + body
            Ok(response)
        }
        Err(e) => {
            tracing::error!("Failed to read index.html: {}", e);
            let response = (StatusCode::NOT_FOUND, "404 - index.html not found").into_response();
            Ok(response)
        }
    }
}
