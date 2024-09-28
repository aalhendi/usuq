use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::Redirect,
    routing::{delete, get, post},
    Extension, Router,
};
use std::sync::Arc;
use tokio_rusqlite::Connection;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{EntryRequest, EntryResponse};
use crate::{
    db::{create_entry, delete_entry_by_slug, get_entry_by_slug},
    middleware::api_key_auth,
};

type Database = Extension<Arc<Connection>>;
type AppResult<T> = Result<T, AppError>;
type AppJsonResult<T> = AppResult<Json<T>>;

pub fn create_route() -> Router {
    let protected_routes = Router::new()
        .route("/", post(handle_entry_post))
        .route("/:slug", delete(handle_entry_delete))
        .route_layer(axum::middleware::from_fn(api_key_auth));

    let public_routes = Router::new()
        .route("/", get(handle_index_get))
        .route("/:slug", get(handle_entry_get));

    public_routes.merge(protected_routes)
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

    let entry = create_entry(&db, id, input_url.to_string(), input.slug).await?;
    Ok(Json(entry))
}

async fn handle_entry_get(db: Database, Path(slug): Path<String>) -> AppResult<Redirect> {
    let url = get_entry_by_slug(&db, slug).await?;
    Ok(Redirect::to(&url))
}

async fn handle_entry_delete(db: Database, Path(slug): Path<String>) -> AppResult<StatusCode> {
    delete_entry_by_slug(&db, slug).await?;
    Ok(StatusCode::OK)
}
