use axum::{
    body::Body,
    http::{Request, Response},
    middleware::Next,
    response::IntoResponse,
};
use uuid::Uuid;
use tracing::info;

pub async fn correlation_id_middleware(
    req: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    let header_name = "x-correlation-id";
    
    let correlation_id = req
        .headers()
        .get(header_name)
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    info!(
        correlation_id = %correlation_id,
        method = %req.method(),
        uri = %req.uri(),
        "Request received"
    );

    let response = next.run(req).await;

    let (mut parts, body) = response.into_parts();
    parts.headers.insert(
        header_name,
        correlation_id.parse().unwrap(),
    );
    
    Response::from_parts(parts, body)
}
