use serde::Serialize;

use axum::Json;

#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    message: String,
}

pub async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        message: "FilePi Rust server is running".to_string(),
    })
}
