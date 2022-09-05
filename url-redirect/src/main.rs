use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{extract::Extension, Router};
use std::sync::Arc;
use std::net::SocketAddr;

mod prisma;
mod routes;

#[tokio::main]
// TODO: Remove unwraps and do real error handling
// TODO: Check for bad urls
// TODO: Add tests

async fn main() {
    let client = Arc::new(prisma::new_client().await.unwrap());

    let app = Router::new()
        .nest("/", routes::create_route())
        .layer(Extension(client));

    // fallback service for handling routes to unknown paths
    let app = app.fallback(axum::routing::get(handle_404));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handle_404() -> impl IntoResponse{
    (StatusCode::NOT_FOUND, "lol, not found.")
}
