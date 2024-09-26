use axum::{extract::Extension, http::StatusCode, response::IntoResponse, Router};
use std::{path::Path, sync::Arc};
use tokio::net::TcpListener;
use tokio_rusqlite::Connection;

mod routes;

// TODO: Add tests

async fn initialize_db(conn: &Connection) -> Result<(), tokio_rusqlite::Error> {
    conn.call(|c| {
        match c.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS Url (
                id TEXT PRIMARY KEY,
                createdAt DATETIME DEFAULT CURRENT_TIMESTAMP,
                url TEXT NOT NULL,
                slug TEXT UNIQUE NOT NULL
            );
            
            CREATE INDEX IF NOT EXISTS idx_slug ON Url(slug);
            ",
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(tokio_rusqlite::Error::Rusqlite(e)),
        }
    })
    .await?;

    Ok(())
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let db_file = "../dev.db";

    if !Path::new(db_file).exists() {
        println!("Database file does not exist. It will be created.");
    } else {
        println!("Using existing database file.");
    }

    let db = Arc::new(Connection::open(db_file).await?);

    println!("Initializing database at {}", db_file);
    initialize_db(&db).await?;
    println!("Database initialized successfully");

    let app = Router::new()
        .nest("/", routes::create_route())
        .layer(Extension(Arc::clone(&db)));

    // fallback service for handling routes to unknown paths
    let app = app.fallback(axum::routing::get(handle_404));

    let listener = TcpListener::bind("127.0.0.1:8000").await?;

    println!("Starting server on {addr}", addr = listener.local_addr()?);

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
        panic!("Application error: {e}");
    }
}
