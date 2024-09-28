use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response, Extension,
};
use secrecy::ExposeSecret;

use crate::configuration::Settings;

pub async fn api_key_auth(
    Extension(config): Extension<Arc<Settings>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let api_key = req
        .headers()
        .get("X-API-Key")
        .and_then(|header| header.to_str().ok());

    match api_key {
        Some(key) if key == config.application.api_key.expose_secret() => Ok(next.run(req).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}
