use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use crate::metrics;

/// Middleware to record API metrics
pub async fn track_metrics(
    req: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    
    let response = next.run(req).await;
    
    let duration = start.elapsed();
    let status = response.status().as_u16();
    
    // Record metrics
    metrics::record_api_request(&method, &path, status, duration);
    
    response
}