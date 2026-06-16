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

use miette::Diagnostic;
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Snafu, Diagnostic)]
#[snafu(visibility(pub(crate)))]
pub enum ConfigError {
    #[snafu(display("could not access config at {}", path.display()))]
    #[diagnostic(code(xevion::cli::config::io))]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("config at {} is not valid TOML", path.display()))]
    #[diagnostic(code(xevion::cli::config::parse))]
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },

    #[snafu(display("could not serialize config"))]
    #[diagnostic(code(xevion::cli::config::serialize))]
    Serialize { source: toml::ser::Error },

    #[snafu(display("could not determine a config directory"))]
    #[diagnostic(
        code(xevion::cli::config::no_dir),
        help("set XEVION_CONFIG=/path/to/config.toml or pass --config <file>")
    )]
    NoConfigDir,

    /// No API target could be resolved (none named, none default).
    #[snafu(display("no API target selected"))]
    #[diagnostic(code(xevion::cli::config::no_target), help("{help}"))]
    NoTarget { help: String },

    /// A `--api <name>` was given but no such entry exists.
    #[snafu(display("no API target named '{name}'"))]
    #[diagnostic(code(xevion::cli::config::unknown_target), help("{help}"))]
    UnknownTarget { name: String, help: String },

    /// The named target exists but has no token yet.
    #[snafu(display("API target '{name}' is not authorized"))]
    #[diagnostic(
        code(xevion::cli::config::unauthorized),
        help("authorize it: xevion login --api {name}")
    )]
    NotAuthenticated { name: String },

    /// A rename collides with an existing target name.
    #[snafu(display("a target named '{name}' already exists"))]
    #[diagnostic(code(xevion::cli::config::target_exists))]
    TargetExists { name: String },
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
/// platform config directory. Errors (rather than silently falling back to the
/// working directory) when no config dir can be determined.
pub fn resolve_path(override_path: Option<&str>) -> Result<PathBuf, ConfigError> {
    if let Some(p) = override_path {
        return Ok(PathBuf::from(p));
    }
    if let Ok(p) = std::env::var("XEVION_CONFIG") {
        return Ok(PathBuf::from(p));
    }
    directories::ProjectDirs::from("dev", "xevion", "xevion")
        .map(|dirs| dirs.config_dir().join("config.toml"))
        .ok_or(ConfigError::NoConfigDir)
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
        let content = std::fs::read_to_string(&path).context(IoSnafu { path: path.clone() })?;
        let mut config: Self =
            toml::from_str(&content).context(ParseSnafu { path: path.clone() })?;
        config.path = path;
        Ok(config)
    }

    /// Serialize and write the config, creating parent dirs and locking it to the
    /// owner (`0600`) since it holds tokens.
    pub fn save(&self) -> Result<(), ConfigError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).context(IoSnafu {
                path: parent.to_path_buf(),
            })?;
        }
        let content = toml::to_string_pretty(self).context(SerializeSnafu)?;
        std::fs::write(&self.path, content).context(IoSnafu {
            path: self.path.clone(),
        })?;
        self.restrict_permissions()?;
        Ok(())
    }

    #[cfg(unix)]
    fn restrict_permissions(&self) -> Result<(), ConfigError> {
        use std::os::unix::fs::PermissionsExt;
        let path = || self.path.clone();
        let mut perms = std::fs::metadata(&self.path)
            .context(IoSnafu { path: path() })?
            .permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(&self.path, perms).context(IoSnafu { path: path() })?;
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
    /// requests. Errors if no target resolves or it has no token. The error
    /// `help` adapts to what the user actually has configured.
    pub fn resolve(&self, override_name: Option<&str>) -> Result<(String, ApiEntry), ConfigError> {
        let name = self
            .target_name(override_name)
            .ok_or_else(|| ConfigError::NoTarget {
                help: self.no_target_help(),
            })?;
        let entry = self
            .api
            .get(&name)
            .cloned()
            .ok_or_else(|| ConfigError::UnknownTarget {
                name: name.clone(),
                help: self.unknown_target_help(),
            })?;
        if entry.token.is_none() {
            return Err(ConfigError::NotAuthenticated { name });
        }
        Ok((name, entry))
    }

    /// Guidance for the no-target case: point at an existing target if there is
    /// one, otherwise the first-run `login` invocation.
    fn no_target_help(&self) -> String {
        match self.names().next() {
            Some(existing) => format!(
                "select a target with --api <name> (e.g. --api {existing}) or set a default: xevion targets use {existing}"
            ),
            None => "authorize a target first: xevion login --api <name> --url <url>".to_string(),
        }
    }

    /// Guidance for an unknown `--api <name>`: list what's configured.
    fn unknown_target_help(&self) -> String {
        let names: Vec<&str> = self.names().collect();
        if names.is_empty() {
            "no targets are configured yet: xevion login --api <name> --url <url>".to_string()
        } else {
            format!("configured targets: {}", names.join(", "))
        }
    }

    /// Look up just the URL for a target without requiring a token. Used by login.
    pub fn url_for(&self, name: &str) -> Option<String> {
        self.api.get(name).map(|e| e.url.clone())
    }

    /// True when a target with this name exists.
    pub fn has_target(&self, name: &str) -> bool {
        self.api.contains_key(name)
    }

    /// Whether a target currently holds a token.
    pub fn is_authorized(&self, name: &str) -> bool {
        self.api.get(name).is_some_and(|e| e.token.is_some())
    }

    /// Iterator over configured target names.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.api.keys().map(String::as_str)
    }

    /// Upsert a target's URL and token, making it the default if none is set.
    pub fn set_target(&mut self, name: &str, url: String, token: Option<String>) {
        self.api.insert(name.to_string(), ApiEntry { url, token });
        if self.default.is_none() {
            self.default = Some(name.to_string());
        }
    }

    /// Make `name` the default target. Errors if it isn't configured.
    pub fn set_default(&mut self, name: &str) -> Result<(), ConfigError> {
        if !self.has_target(name) {
            return Err(ConfigError::UnknownTarget {
                name: name.to_string(),
                help: self.unknown_target_help(),
            });
        }
        self.default = Some(name.to_string());
        Ok(())
    }

    /// Remove a target entirely, clearing the default if it pointed here.
    pub fn remove_target(&mut self, name: &str) -> Result<ApiEntry, ConfigError> {
        let entry = self
            .api
            .remove(name)
            .ok_or_else(|| ConfigError::UnknownTarget {
                name: name.to_string(),
                help: self.unknown_target_help(),
            })?;
        if self.default.as_deref() == Some(name) {
            self.default = None;
        }
        Ok(entry)
    }

    /// Rename a target, preserving its URL/token and default status. Errors if
    /// `from` is missing or `to` already exists.
    pub fn rename_target(&mut self, from: &str, to: &str) -> Result<(), ConfigError> {
        if !self.has_target(from) {
            return Err(ConfigError::UnknownTarget {
                name: from.to_string(),
                help: self.unknown_target_help(),
            });
        }
        if self.has_target(to) {
            return Err(ConfigError::TargetExists {
                name: to.to_string(),
            });
        }
        let entry = self.api.remove(from).expect("checked above");
        self.api.insert(to.to_string(), entry);
        if self.default.as_deref() == Some(from) {
            self.default = Some(to.to_string());
        }
        Ok(())
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
            Err(ConfigError::NotAuthenticated { .. })
        ));
    }

    #[test]
    fn resolve_errors_for_unknown_override() {
        let mut config = Config::default();
        config.set_target("local", "http://localhost:10237".into(), Some("t1".into()));
        assert!(matches!(
            config.resolve(Some("nope")),
            Err(ConfigError::UnknownTarget { .. })
        ));
    }

    #[test]
    fn resolve_errors_with_no_targets_at_all() {
        let config = Config::default();
        assert!(matches!(
            config.resolve(None),
            Err(ConfigError::NoTarget { .. })
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

    #[test]
    fn set_default_requires_existing_target() {
        let mut config = Config::default();
        config.set_target("local", "http://localhost:10237".into(), Some("t1".into()));
        config.set_target("prod", "https://xevion.dev".into(), Some("t2".into()));

        config.set_default("prod").unwrap();
        assert_eq!(config.default.as_deref(), Some("prod"));
        assert!(matches!(
            config.set_default("ghost"),
            Err(ConfigError::UnknownTarget { .. })
        ));
    }

    #[test]
    fn remove_target_clears_dangling_default() {
        let mut config = Config::default();
        config.set_target("local", "http://localhost:10237".into(), Some("t1".into()));
        config.remove_target("local").unwrap();
        assert!(config.default.is_none());
        assert!(!config.has_target("local"));
        assert!(matches!(
            config.remove_target("local"),
            Err(ConfigError::UnknownTarget { .. })
        ));
    }

    #[test]
    fn rename_target_preserves_default_and_rejects_collision() {
        let mut config = Config::default();
        config.set_target("local", "http://localhost:10237".into(), Some("t1".into()));
        config.set_target("prod", "https://xevion.dev".into(), Some("t2".into()));

        config.rename_target("local", "dev").unwrap();
        assert!(config.has_target("dev"));
        assert!(!config.has_target("local"));
        assert_eq!(config.default.as_deref(), Some("dev"));

        assert!(matches!(
            config.rename_target("dev", "prod"),
            Err(ConfigError::TargetExists { .. })
        ));
    }
}
