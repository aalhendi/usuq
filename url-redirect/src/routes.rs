use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::{IntoResponse, Response, Redirect},
    routing::get,
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
/l/:slug => PUT, DELETE

*/
pub fn create_route() -> Router {
    Router::new()
        .route("/", get(handle_index_get).post(handle_entry_post))
        .route(
            "/l/:slug",
            get(handle_entry_get).delete(handle_entry_delete),
        )
}

async fn handle_index_get(_db: Database) -> AppResult<Json<String>> {
    Ok(Json::from("{value: helloworld}".to_string()))
}

async fn handle_entry_post(
    db: Database,
    Json(input): Json<EntryRequest>,
) -> AppJsonResult<url::Data> {
    let data = db
        .url()
        .create(input.url, input.slug, vec![])
        .exec()
        .await?;

    Ok(Json::from(data))
}

async fn handle_entry_get(
    db: Database,
    Path(slug): Path<String>,
) -> AppResult<Redirect> {
    let entry = db
        .url()
        .find_unique(prisma::url::slug::equals(slug)) // Query to execute
        .exec()
        .await?;

    let entry_data = match entry {
        Some(data)=>data,
        None => panic!("Couldn't find data in entry")
    };

    Ok(Redirect::to(entry_data.url.as_str()))
}

async fn handle_entry_delete(db: Database, Path(slug): Path<String>) -> AppResult<StatusCode> {
    db.url()
        .delete(url::slug::equals(slug))
        .exec()
        .await?;

    Ok(StatusCode::OK)
}

enum AppError {
    PrismaError(QueryError),
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

// This centralizes all differents errors from our app in one place
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::PrismaError(error) if error.is_prisma_error::<UniqueKeyViolation>() => {
                StatusCode::CONFLICT
            }
            AppError::PrismaError(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound => StatusCode::NOT_FOUND,
        };

        status.into_response()
    }
}
