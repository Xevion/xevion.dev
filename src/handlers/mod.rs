pub mod assets;
pub mod auth;
pub mod cli_auth;
pub mod content;
pub mod events;
pub mod health;
pub mod icons;
pub mod media;
pub mod projects;
pub mod sessions;
pub mod settings;
pub mod tags;

pub use assets::*;
pub use auth::*;
pub use cli_auth::*;
pub use content::*;
pub use events::*;
pub use health::*;
pub use icons::*;
pub use media::*;
pub use projects::*;
pub use sessions::*;
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
    pub status: crate::db::ProjectStatus,
    pub github_repo: Option<String>,
    pub demo_url: Option<String>,
    pub tag_ids: Vec<String>,
    /// `TipTap` document JSON. Null/absent means the project has no detail page.
    #[serde(default)]
    pub detail_content: Option<serde_json::Value>,
    /// Authored primary label ("CLI Tool", "Web App", …).
    #[serde(default)]
    pub project_type: Option<String>,
    /// Closed-source flag (orthogonal to `status`).
    #[serde(default)]
    pub source_closed: bool,
    /// Authored CLI-hero transcript.
    #[serde(default)]
    pub terminal_cast: Option<crate::db::TerminalCast>,
    /// Explicit per-project accent (hex).
    #[serde(default)]
    pub accent_color: Option<String>,
    /// Curated related project IDs, in authored order (full replace).
    #[serde(default)]
    pub related_ids: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectRequest {
    pub name: String,
    pub slug: Option<String>,
    pub short_description: String,
    pub status: crate::db::ProjectStatus,
    pub github_repo: Option<String>,
    pub demo_url: Option<String>,
    pub tag_ids: Vec<String>,
    /// `TipTap` document JSON. PUT semantics: this is the full new value, so null
    /// (or absent) clears the detail page. The admin form always sends the current
    /// editor state (null when empty); the CLI echoes the existing value to preserve it.
    #[serde(default)]
    pub detail_content: Option<serde_json::Value>,
    /// Authored primary label ("CLI Tool", "Web App", …). PUT replace.
    #[serde(default)]
    pub project_type: Option<String>,
    /// Closed-source flag (orthogonal to `status`).
    #[serde(default)]
    pub source_closed: bool,
    /// Authored CLI-hero transcript. PUT replace (null clears it).
    #[serde(default)]
    pub terminal_cast: Option<crate::db::TerminalCast>,
    /// Explicit per-project accent (hex). PUT replace.
    #[serde(default)]
    pub accent_color: Option<String>,
    /// Curated related project IDs, in authored order (full replace).
    #[serde(default)]
    pub related_ids: Vec<String>,
}
