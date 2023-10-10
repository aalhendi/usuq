use axum::{extract::Extension, http::StatusCode, response::IntoResponse, Router};
use std::net::SocketAddr;
use std::sync::Arc;

mod prisma;
mod routes;

// TODO: Add tests

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let client = Arc::new(prisma::new_client().await?);

    let app = Router::new()
        .nest("/", routes::create_route())
        .layer(Extension(client));

    // fallback service for handling routes to unknown paths
    let app = app.fallback(axum::routing::get(handle_404));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));

    println!("Starting server on {addr}");

    let server = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(handle_signals());

    server.await?;

    Ok(())
}

async fn handle_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Resource not found.")
}

#[cfg(unix)]
async fn handle_signals() {
    use tokio::signal::unix::SignalKind;
    let mut term_signal = tokio::signal::unix::signal(SignalKind::terminate()).unwrap();
    let mut int_signal = tokio::signal::unix::signal(SignalKind::interrupt()).unwrap();
    tokio::select! {
        _ = term_signal.recv() => {
            println!("Received SIGTERM.");
        }
        _ = int_signal.recv() => {
            println!("Received SIGINT.");
        }
    }
}

#[cfg(windows)]
async fn handle_signals() {
    use tokio::signal::ctrl_c;
    ctrl_c().await.unwrap();
    println!("Received CTRL+C.");
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        panic!("Application error: {e}");
    }
}