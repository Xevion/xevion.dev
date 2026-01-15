pub mod assets;
pub mod auth;
pub mod health;
pub mod icons;
pub mod media;
pub mod projects;
pub mod settings;
pub mod tags;

// Re-export handlers for easier imports
pub use assets::*;
pub use auth::*;
pub use health::*;
pub use icons::*;
pub use media::*;
pub use projects::*;
pub use settings::*;
pub use tags::*;

// Request/Response types used by handlers

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTagRequest {
    pub name: String,
    pub slug: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTagRequest {
    pub name: String,
    pub slug: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddProjectTagRequest {
    pub tag_id: String,
}
