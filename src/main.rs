use axum::{extract::Extension, http::StatusCode, response::IntoResponse, Router};
use configuration::Settings;
use std::{path::Path, sync::Arc};
use tokio::net::TcpListener;
use tokio_rusqlite::Connection;

mod configuration;
mod db;
mod errors;
mod models;
mod routes;

use crate::db::initialize_db;
use crate::routes::create_route;

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = Settings::new()?;

    if !Path::new(&config.database.path).exists() {
        println!("Database file does not exist. It will be created.");
    } else {
        println!("Using existing database file.");
    }

    let db = Arc::new(Connection::open(&config.database.path).await?);

    println!(
        "Initializing database at {db_path}",
        db_path = config.database.path
    );
    initialize_db(&db).await?;
    println!("Database initialized successfully");

    let app = Router::new()
        .nest("/", create_route())
        .layer(Extension(Arc::clone(&db)));

    let app = app.fallback(axum::routing::get(handle_404));

    let addr = format!(
        "{host}:{port}",
        host = config.application.host,
        port = config.application.port
    );
    let listener = TcpListener::bind(&addr).await?;

    println!("Starting server on {addr}");

    let server = axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(handle_signals(Arc::clone(&db)));

    server.await?;

    Ok(())
}

async fn handle_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Resource not found.")
}

#[cfg(unix)]
async fn handle_signals(conn: Arc<Connection>) {
    use tokio::signal::unix::SignalKind;
    let mut term_signal = tokio::signal::unix::signal(SignalKind::terminate()).unwrap();
    let mut int_signal = tokio::signal::unix::signal(SignalKind::interrupt()).unwrap();
    tokio::select! {
        _ = term_signal.recv() => {
            Arc::try_unwrap(conn).unwrap().close().await.expect("Failed to close DB connection.");
            println!("Received SIGTERM.");
        }
        _ = int_signal.recv() => {
            Arc::try_unwrap(conn).unwrap().close().await.expect("Failed to close DB connection.");
            println!("Received SIGINT.");
        }
    }
}

#[cfg(windows)]
async fn handle_signals(conn: Arc<Connection>) {
    use tokio::signal::ctrl_c;
    Arc::try_unwrap(conn)
        .unwrap()
        .close()
        .await
        .expect("Failed to close DB connection.");
    ctrl_c().await.unwrap();
    println!("Received CTRL+C.");
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Application error: {e}");
        std::process::exit(1);
    }
}
