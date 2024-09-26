use std::sync::Arc;

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::MethodRouter,
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio_rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

type Database = Extension<Arc<Connection>>;
type AppResult<T> = Result<T, AppError>;
type AppJsonResult<T> = AppResult<Json<T>>;

#[derive(Deserialize)]
struct EntryRequest {
    url: String,
    slug: String,
}

#[derive(Serialize)]
struct EntryResponse {
    id: String,
    url: String,
    slug: String,
    created_at: String,
}

/*

/ => GET, POST
/:slug => PUT, DELETE

*/
pub fn create_route() -> Router {
    let route_root = MethodRouter::new()
        .get(handle_index_get)
        .post(handle_entry_post);

    let route_slug = MethodRouter::new()
        .get(handle_entry_get)
        .delete(handle_entry_delete);

    Router::new()
        .route("/", route_root)
        .route("/:slug", route_slug)
}

async fn handle_index_get() -> AppResult<Json<String>> {
    Ok(Json::from("{value: who dis}".to_string()))
}

async fn handle_entry_post(
    db: Database,
    Json(input): Json<EntryRequest>,
) -> AppJsonResult<EntryResponse> {
    let input_url = ::url::Url::parse(&input.url)?;
    let id = Uuid::new_v4().to_string();

    let entry = db
        .call(move |conn| {
            conn.execute(
                "INSERT INTO Url (id, url, slug) VALUES (?1, ?2, ?3)",
                params![id, input_url.to_string(), input.slug],
            )?;

            let entry = conn.query_row(
                "SELECT id, url, slug, createdAt FROM Url WHERE id = ?1",
                params![id],
                |row| {
                    Ok(EntryResponse {
                        id: row.get(0)?,
                        url: row.get(1)?,
                        slug: row.get(2)?,
                        created_at: row.get(3)?,
                    })
                },
            )?;

            Ok(entry)
        })
        .await?;

    Ok(Json(entry))
}

async fn handle_entry_get(db: Database, Path(slug): Path<String>) -> AppResult<Redirect> {
    let url = db
        .call(move |conn| {
            let url_maybe = conn
                .query_row(
                    "SELECT url FROM Url WHERE slug = ?1",
                    params![slug],
                    |row| row.get::<_, String>(0),
                )
                .optional()?;

            Ok(url_maybe)
        })
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Redirect::to(&url))
}

async fn handle_entry_delete(db: Database, Path(slug): Path<String>) -> AppResult<StatusCode> {
    let rows_affected = db
        .call(move |conn| {
            let rows_affected = conn.execute("DELETE FROM Url WHERE slug = ?1", params![slug])?;
            Ok(rows_affected)
        })
        .await?;

    if rows_affected == 0 {
        Err(AppError::NotFound)
    } else {
        Ok(StatusCode::OK)
    }
}

#[derive(Error, Debug)]
enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] tokio_rusqlite::Error),
    #[error("URL parse error: {0}")]
    UrlParseError(#[from] ::url::ParseError),
    #[error("Not found")]
    NotFound,
}


// This centralizes all differents errors from our app in one place
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::DatabaseError(ref e)
                if e.to_string().contains("UNIQUE constraint failed") =>
            {
                (StatusCode::CONFLICT, "Slug already exists".to_string())
            }
            AppError::DatabaseError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not Found".to_string()),
            AppError::UrlParseError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
        };

        (status, message).into_response()
    }
}
