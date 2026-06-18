// The handler implementations are server-only (they depend on `AppState`, asset
// serving, the proxy, …). The request DTOs further down are shared with the
// `xevion` CLI and stay ungated.
#[cfg(feature = "server")]
pub mod assets;
#[cfg(feature = "server")]
pub mod auth;
#[cfg(feature = "server")]
pub mod cli_auth;
#[cfg(feature = "server")]
pub mod content;
#[cfg(feature = "server")]
pub mod events;
#[cfg(feature = "server")]
pub mod health;
#[cfg(feature = "server")]
pub mod icons;
#[cfg(feature = "server")]
pub mod media;
#[cfg(feature = "server")]
pub mod projects;
#[cfg(feature = "server")]
pub mod seo;
#[cfg(feature = "server")]
pub mod sessions;
#[cfg(feature = "server")]
pub mod settings;
#[cfg(feature = "server")]
pub mod tags;

#[cfg(feature = "server")]
pub use assets::*;
#[cfg(feature = "server")]
pub use auth::*;
#[cfg(feature = "server")]
pub use cli_auth::*;
#[cfg(feature = "server")]
pub use content::*;
#[cfg(feature = "server")]
pub use events::*;
#[cfg(feature = "server")]
pub use health::*;
#[cfg(feature = "server")]
pub use icons::*;
#[cfg(feature = "server")]
pub use media::*;
#[cfg(feature = "server")]
pub use projects::*;
#[cfg(feature = "server")]
pub use seo::*;
#[cfg(feature = "server")]
pub use sessions::*;
#[cfg(feature = "server")]
pub use settings::*;
#[cfg(feature = "server")]
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
    /// Overall public visibility, independent of activity `status`.
    #[serde(default)]
    pub hidden: bool,
    pub github_repo: Option<String>,
    pub demo_url: Option<String>,
    pub tag_ids: Vec<String>,
    /// `TipTap` document JSON. Null/absent means the project has no detail page.
    #[serde(default)]
    pub detail_content: Option<serde_json::Value>,
    /// Authored primary label ("CLI Tool", "Web App", …).
    #[serde(default)]
    pub project_type: Option<String>,
    /// Source-is-private flag (orthogonal to `status`): hides repo links while
    /// the repo keeps syncing.
    #[serde(default)]
    pub private: bool,
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
    /// Overall public visibility, independent of activity `status`.
    #[serde(default)]
    pub hidden: bool,
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
    /// Source-is-private flag (orthogonal to `status`): hides repo links while
    /// the repo keeps syncing.
    #[serde(default)]
    pub private: bool,
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
