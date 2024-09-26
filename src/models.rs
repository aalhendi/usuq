use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct EntryRequest {
    pub url: String,
    pub slug: String,
}

#[derive(Serialize)]
pub struct EntryResponse {
    pub id: String,
    pub url: String,
    pub slug: String,
    pub created_at: String,
}
