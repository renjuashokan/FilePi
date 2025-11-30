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

    let content_type = parts
        .headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let is_binary = content_type.starts_with("image/")
        || content_type.starts_with("video/")
        || content_type.starts_with("application/octet-stream")
        || content_type.starts_with("multipart/form-data");

    let (body_str, request) = if is_binary {
        (
            format!("<binary data: {}>", content_type),
            Request::from_parts(parts, body),
        )
    } else {
        let bytes = body.collect().await.unwrap_or_default().to_bytes();
        let mut body_str = String::from_utf8_lossy(&bytes).to_string();
        
        if body_str.len() > 100 {
            body_str.truncate(100);
            body_str.push_str("... (truncated)");
        }

        (
            body_str,
            Request::from_parts(parts, Body::from(bytes)),
        )
    };

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
