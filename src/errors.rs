use axum::{http::StatusCode, response::IntoResponse, response::Response};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] tokio_rusqlite::Error),
    #[error("URL parse error: {0}")]
    UrlParseError(#[from] ::url::ParseError),
    #[error("Not found")]
    NotFound,
}

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
