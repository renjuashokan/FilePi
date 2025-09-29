use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;
use tracing::info;

pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();
    let query = uri.query().unwrap_or("").to_string();

    // Process the request
    let response = next.run(request).await;

    // Calculate latency
    let latency = start.elapsed();
    let status = response.status();

    // Log the request
    info!(
        method = %method,
        path = %path,
        query = %query,
        status = %status.as_u16(),
        latency = ?latency,
        "HTTP request"
    );

    response
}
