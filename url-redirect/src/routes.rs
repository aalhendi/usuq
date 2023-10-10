use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::MethodRouter,
    Extension, Router,
};
use prisma_client_rust::{
    prisma_errors::query_engine::{RecordNotFound, UniqueKeyViolation},
    QueryError,
};

use serde::Deserialize;

use crate::prisma::{self, url};

type Database = Extension<std::sync::Arc<prisma::PrismaClient>>;
type AppResult<T> = Result<T, AppError>;
type AppJsonResult<T> = AppResult<Json<T>>;

// Define all your requests schema
#[derive(Deserialize)]
struct EntryRequest {
    url: String,
    slug: String,
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
) -> AppJsonResult<url::Data> {
    let input_url = ::url::Url::parse(&input.url)?;

    let data = db
        .url()
        .create(input_url.into(), input.slug, vec![])
        .exec()
        .await?;

    Ok(Json::from(data))
}

async fn handle_entry_get(db: Database, Path(slug): Path<String>) -> AppResult<Redirect> {
    let entry = db
        .url()
        .find_unique(prisma::url::slug::equals(slug)) // Query to execute
        .exec()
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Redirect::to(&entry.url))
}

async fn handle_entry_delete(db: Database, Path(slug): Path<String>) -> AppResult<StatusCode> {
    db.url().delete(url::slug::equals(slug)).exec().await?;

    Ok(StatusCode::OK)
}

enum AppError {
    PrismaError(QueryError),
    UrlParseError(::url::ParseError),
    NotFound,
}

impl From<QueryError> for AppError {
    fn from(error: QueryError) -> Self {
        match error {
            e if e.is_prisma_error::<RecordNotFound>() => AppError::NotFound,
            e => AppError::PrismaError(e),
        }
    }
}

impl From<::url::ParseError> for AppError {
    fn from(error: ::url::ParseError) -> Self {
        AppError::UrlParseError(error)
    }
}

// This centralizes all differents errors from our app in one place
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::PrismaError(error) if error.is_prisma_error::<UniqueKeyViolation>() => {
                (StatusCode::CONFLICT, error.to_string())
            }
            AppError::PrismaError(error) => (StatusCode::BAD_REQUEST, error.to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not Found".to_string()),
            AppError::UrlParseError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
        };

        (status, message).into_response()
    }
}
