pub mod assets;
pub mod auth;
pub mod events;
pub mod health;
pub mod icons;
pub mod media;
pub mod projects;
pub mod settings;
pub mod tags;

pub use assets::*;
pub use auth::*;
pub use events::*;
pub use health::*;
pub use icons::*;
pub use media::*;
pub use projects::*;
pub use settings::*;
pub use tags::*;

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

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectRequest {
    pub name: String,
    pub slug: Option<String>,
    pub short_description: String,
    pub description: String,
    pub status: crate::db::ProjectStatus,
    pub github_repo: Option<String>,
    pub demo_url: Option<String>,
    pub tag_ids: Vec<String>,
    /// `TipTap` document JSON. Null/absent means the project has no detail page.
    #[serde(default)]
    pub detail_content: Option<serde_json::Value>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectRequest {
    pub name: String,
    pub slug: Option<String>,
    pub short_description: String,
    pub description: String,
    pub status: crate::db::ProjectStatus,
    pub github_repo: Option<String>,
    pub demo_url: Option<String>,
    pub tag_ids: Vec<String>,
    /// `TipTap` document JSON. PUT semantics: this is the full new value, so null
    /// (or absent) clears the detail page. The admin form always sends the current
    /// editor state (null when empty); the CLI echoes the existing value to preserve it.
    #[serde(default)]
    pub detail_content: Option<serde_json::Value>,
}
