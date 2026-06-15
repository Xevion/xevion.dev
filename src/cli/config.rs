//! TOML-backed CLI config: a set of named API targets, each with a base URL and
//! (once authenticated) a long-lived bearer token, plus a default selection.
//!
//! ```toml
//! default = "local"
//!
//! [api.local]
//! url = "http://localhost:10237"
//! token = "xev_..."
//!
//! [api.production]
//! url = "https://xevion.dev"
//! token = "xev_..."
//! ```
//!
//! Lives under the platform config dir (`~/.config/xevion/config.toml` on Linux),
//! written `0600` since it holds tokens.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// The conventional default name for a local development target.
pub const DEFAULT_API_NAME: &str = "local";

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Parse(String),
    /// No API target could be resolved (none named, none default).
    NoTarget(String),
    /// The named target exists but has no token yet.
    NotAuthenticated(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "Config I/O error: {e}"),
            Self::Parse(e) => write!(f, "Config parse error: {e}"),
            Self::NoTarget(msg) | Self::NotAuthenticated(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// A single named API target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEntry {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    /// Name of the target used when `--api` is omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub api: BTreeMap<String, ApiEntry>,

    /// Path this config was loaded from; not serialized.
    #[serde(skip)]
    path: PathBuf,
}

/// Resolve the config file path: `--config`/`XEVION_CONFIG` override, else the
/// platform config directory.
pub fn resolve_path(override_path: Option<&str>) -> PathBuf {
    if let Some(p) = override_path {
        return PathBuf::from(p);
    }
    if let Ok(p) = std::env::var("XEVION_CONFIG") {
        return PathBuf::from(p);
    }
    directories::ProjectDirs::from("dev", "xevion", "xevion").map_or_else(
        || PathBuf::from("xevion.toml"),
        |dirs| dirs.config_dir().join("config.toml"),
    )
}

impl Config {
    /// Load the config from `path`, returning an empty config if it doesn't exist.
    pub fn load(path: PathBuf) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Ok(Self {
                path,
                ..Self::default()
            });
        }
        let content = std::fs::read_to_string(&path)?;
        let mut config: Self =
            toml::from_str(&content).map_err(|e| ConfigError::Parse(e.to_string()))?;
        config.path = path;
        Ok(config)
    }

    /// Serialize and write the config, creating parent dirs and locking it to the
    /// owner (`0600`) since it holds tokens.
    pub fn save(&self) -> Result<(), ConfigError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content =
            toml::to_string_pretty(self).map_err(|e| ConfigError::Parse(e.to_string()))?;
        std::fs::write(&self.path, content)?;
        self.restrict_permissions()?;
        Ok(())
    }

    #[cfg(unix)]
    fn restrict_permissions(&self) -> Result<(), ConfigError> {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&self.path)?.permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(&self.path, perms)?;
        Ok(())
    }

    #[cfg(not(unix))]
    fn restrict_permissions(&self) -> Result<(), ConfigError> {
        Ok(())
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// The target name to use given an optional `--api` override: the override,
    /// else the configured default.
    fn target_name(&self, override_name: Option<&str>) -> Option<String> {
        override_name
            .map(str::to_string)
            .or_else(|| self.default.clone())
    }

    /// Resolve a fully-authenticated target (name + url + token) for issuing
    /// requests. Errors if no target resolves or it has no token.
    pub fn resolve(&self, override_name: Option<&str>) -> Result<(String, ApiEntry), ConfigError> {
        let name = self.target_name(override_name).ok_or_else(|| {
            ConfigError::NoTarget(
                "No API target configured. Run 'xevion api login --url <url>' first.".to_string(),
            )
        })?;
        let entry = self.api.get(&name).cloned().ok_or_else(|| {
            ConfigError::NoTarget(format!(
                "No API target named '{name}'. Run 'xevion api login --api {name} --url <url>'."
            ))
        })?;
        if entry.token.is_none() {
            return Err(ConfigError::NotAuthenticated(format!(
                "API target '{name}' has no token. Run 'xevion api login --api {name}'."
            )));
        }
        Ok((name, entry))
    }

    /// Look up just the URL for a target without requiring a token. Used by login.
    pub fn url_for(&self, name: &str) -> Option<String> {
        self.api.get(name).map(|e| e.url.clone())
    }

    /// Upsert a target's URL and token, making it the default if none is set.
    pub fn set_target(&mut self, name: &str, url: String, token: Option<String>) {
        self.api.insert(name.to_string(), ApiEntry { url, token });
        if self.default.is_none() {
            self.default = Some(name.to_string());
        }
    }

    /// Clear a target's token (logout) without removing the entry.
    pub fn clear_token(&mut self, name: &str) {
        if let Some(entry) = self.api.get_mut(name) {
            entry.token = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_target_adopts_first_as_default() {
        let mut config = Config::default();
        config.set_target("local", "http://localhost:10237".into(), Some("t1".into()));
        assert_eq!(config.default.as_deref(), Some("local"));

        // A second target doesn't steal the default.
        config.set_target("production", "https://xevion.dev".into(), Some("t2".into()));
        assert_eq!(config.default.as_deref(), Some("local"));
    }

    #[test]
    fn resolve_prefers_override_then_default() {
        let mut config = Config::default();
        config.set_target("local", "http://localhost:10237".into(), Some("t1".into()));
        config.set_target("production", "https://xevion.dev".into(), Some("t2".into()));

        // Default is `local` (first added).
        let (name, _) = config.resolve(None).unwrap();
        assert_eq!(name, "local");

        // Override wins.
        let (name, entry) = config.resolve(Some("production")).unwrap();
        assert_eq!(name, "production");
        assert_eq!(entry.token.as_deref(), Some("t2"));
    }

    #[test]
    fn resolve_errors_without_token() {
        let mut config = Config::default();
        config.set_target("local", "http://localhost:10237".into(), None);
        assert!(matches!(
            config.resolve(None),
            Err(ConfigError::NotAuthenticated(_))
        ));
    }

    #[test]
    fn resolve_errors_for_unknown_target() {
        let config = Config::default();
        assert!(matches!(
            config.resolve(Some("nope")),
            Err(ConfigError::NoTarget(_))
        ));
    }

    #[test]
    fn clear_token_keeps_entry() {
        let mut config = Config::default();
        config.set_target("local", "http://localhost:10237".into(), Some("t1".into()));
        config.clear_token("local");
        assert_eq!(
            config.url_for("local").as_deref(),
            Some("http://localhost:10237")
        );
        assert!(config.resolve(Some("local")).is_err());
    }
}
