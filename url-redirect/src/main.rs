use axum::{extract::Extension, Router};
use std::sync::Arc;

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

    axum::Server::bind(&"0.0.0.0:8081".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
