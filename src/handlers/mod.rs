pub mod assets;
pub mod auth;
pub mod health;
pub mod projects;
pub mod settings;
pub mod tags;

// Re-export handlers for easier imports
pub use assets::*;
pub use auth::*;
pub use health::*;
pub use projects::*;
pub use settings::*;
pub use tags::*;

// Request/Response types used by handlers

#[derive(serde::Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
    pub slug: Option<String>,
    pub color: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct UpdateTagRequest {
    pub name: String,
    pub slug: Option<String>,
    pub color: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct AddProjectTagRequest {
    pub tag_id: String,
}
