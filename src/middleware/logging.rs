use axum::body::Body;
use axum::{extract::Request, middleware::Next, response::Response};
use http_body_util::BodyExt;
use std::time::Instant;
use tracing::{debug, info};

pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();
    let query = uri.query().unwrap_or("").to_string();

    // Extract and read the body
    let (parts, body) = request.into_parts();
    let bytes = body.collect().await.unwrap_or_default().to_bytes();

    // Convert bytes to string for logging (if it's text)
    let body_str = String::from_utf8_lossy(&bytes);

    // Reconstruct the request with the body
    let request = Request::from_parts(parts, Body::from(bytes.clone()));

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

    debug!(
        body = %body_str,
        "Detailed HTTP request"
    );

    response
}
