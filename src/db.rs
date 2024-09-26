use std::sync::Arc;
use tokio_rusqlite::{params, Connection, OptionalExtension};

use crate::errors::AppError;
use crate::models::EntryResponse;

pub async fn initialize_db(conn: &Connection) -> Result<(), tokio_rusqlite::Error> {
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
    .await
}

pub async fn create_entry(
    db: &Arc<Connection>,
    id: String,
    url: String,
    slug: String,
) -> Result<EntryResponse, AppError> {
    db.call(move |conn| {
        conn.execute(
            "INSERT INTO Url (id, url, slug) VALUES (?1, ?2, ?3)",
            params![id, url, slug],
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
    .await
    .map_err(AppError::DatabaseError)
}

pub async fn get_entry_by_slug(db: &Arc<Connection>, slug: String) -> Result<String, AppError> {
    db.call(move |conn| {
        match conn
            .query_row(
                "SELECT url FROM Url WHERE slug = ?1",
                params![slug],
                |row| row.get::<_, String>(0),
            )
            .optional()
        {
            Ok(maybe_s) => Ok(maybe_s),
            Err(e) => Err(tokio_rusqlite::Error::Rusqlite(e)),
        }
    })
    .await
    .map_err(AppError::DatabaseError)?
    .ok_or(AppError::NotFound)
}

pub async fn delete_entry_by_slug(db: &Arc<Connection>, slug: String) -> Result<(), AppError> {
    let rows_affected = db
        .call(
            move |conn| match conn.execute("DELETE FROM Url WHERE slug = ?1", params![slug]) {
                Ok(n) => Ok(n),
                Err(e) => Err(tokio_rusqlite::Error::Rusqlite(e)),
            },
        )
        .await
        .map_err(AppError::DatabaseError)?;

    if rows_affected == 0 {
        Err(AppError::NotFound)
    } else {
        Ok(())
    }
}
