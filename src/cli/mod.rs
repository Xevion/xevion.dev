// Doc comments here are clap `--help` text, not rustdoc; clap renders backticks
// literally, so don't let `doc_markdown` push CamelCase product names into them.
#![allow(clippy::doc_markdown)]

pub mod api;
pub mod client;
pub mod config;
pub mod error;
pub mod output;
pub mod seed;
pub mod serve;

use clap::{Parser, Subcommand, ValueEnum};

use crate::db::ProjectStatus;

/// `xevion` — API client for managing xevion.dev content remotely.
#[derive(Parser, Debug)]
#[command(name = "xevion")]
#[command(about = "Manage xevion.dev content remotely (projects, tags, settings)")]
#[command(version)]
pub struct ApiArgs {
    /// Named API target from the config (defaults to the config's `default`)
    #[arg(long, env = "XEVION_API", global = true)]
    pub api: Option<String>,

    /// Path to the config file (defaults to the platform config dir)
    #[arg(long, env = "XEVION_CONFIG", global = true)]
    pub config: Option<String>,

    /// Output raw JSON instead of formatted text
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: ApiCommand,
}

#[derive(Subcommand, Debug)]
pub enum ApiCommand {
    /// Authorize this CLI via the browser and save a long-lived token
    Login {
        /// API base URL (required when first authorizing a new target)
        #[arg(long)]
        url: Option<String>,

        /// Device label shown in the approval dialog (defaults to hostname)
        #[arg(long)]
        label: Option<String>,

        /// Print the approval URL instead of opening a browser
        #[arg(long)]
        no_browser: bool,
    },

    /// Revoke and clear the saved token for the selected target
    Logout,

    /// Check current session status
    Session,

    /// List and manage configured API targets
    Targets {
        /// Omit to list targets; otherwise manage them
        #[command(subcommand)]
        command: Option<TargetsCommand>,
    },

    /// Inspect CLI configuration
    #[command(subcommand)]
    Config(ConfigCommand),

    /// Project management
    #[command(subcommand)]
    Projects(ProjectsCommand),

    /// Tag management
    #[command(subcommand)]
    Tags(TagsCommand),

    /// Site settings management
    #[command(subcommand)]
    Settings(SettingsCommand),
}

#[derive(Subcommand, Debug)]
pub enum TargetsCommand {
    /// Make a target the default (used when --api is omitted)
    Use {
        /// Target name
        name: String,
    },

    /// Add a target without authorizing it (set its base URL)
    Add {
        /// Target name
        name: String,

        /// API base URL
        #[arg(long)]
        url: String,
    },

    /// Remove a target
    Rm {
        /// Target name
        name: String,
    },

    /// Rename a target, preserving its URL, token, and default status
    Rename {
        /// Current name
        from: String,

        /// New name
        to: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// Print the resolved config file path
    Path,
}

/// Project lifecycle status, validated at parse time and mirrored to
/// [`ProjectStatus`](crate::db::ProjectStatus).
#[derive(Copy, Clone, Debug, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum StatusArg {
    Active,
    Maintained,
    Archived,
    Hidden,
}

impl From<StatusArg> for ProjectStatus {
    fn from(arg: StatusArg) -> Self {
        match arg {
            StatusArg::Active => Self::Active,
            StatusArg::Maintained => Self::Maintained,
            StatusArg::Archived => Self::Archived,
            StatusArg::Hidden => Self::Hidden,
        }
    }
}

/// clap `value_parser` for hex colors: accepts `#6366F1` or `6366F1`, returns the
/// canonical bare lowercase form. Shared by `--color` (tags) and `--accent`
/// (projects) so both flags behave identically.
pub fn parse_hex_color(raw: &str) -> Result<String, String> {
    // An empty value is the documented "clear this field" sentinel on update
    // commands; pass it through untouched and let the handler drop it.
    if raw.is_empty() {
        return Ok(String::new());
    }
    let hex = raw.strip_prefix('#').unwrap_or(raw);
    if hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
        Ok(hex.to_ascii_lowercase())
    } else {
        Err(format!(
            "'{raw}' is not a 6-digit hex color (e.g. '6366F1' or '#6366F1')"
        ))
    }
}

#[derive(Subcommand, Debug)]
pub enum ProjectsCommand {
    /// List all projects
    List,

    /// Get project details by slug or UUID
    Get {
        /// Project slug or UUID
        #[arg(name = "ref")]
        reference: String,
    },

    /// Create a new project
    Create {
        /// Project name
        name: String,

        /// Short description
        #[arg(short = 's', long)]
        short_desc: String,

        /// URL slug (auto-generated from name if omitted)
        #[arg(long)]
        slug: Option<String>,

        /// Project status
        #[arg(long, value_enum, default_value_t = StatusArg::Active)]
        status: StatusArg,

        /// GitHub repository (e.g., "Xevion/xevion.dev")
        #[arg(long)]
        github_repo: Option<String>,

        /// Demo URL
        #[arg(long)]
        demo_url: Option<String>,

        /// Tags to add (comma-separated, + prefix optional)
        #[arg(short = 't', long)]
        tags: Option<String>,

        /// Accent color hex, e.g. "6366F1" or "#6366F1"
        #[arg(long, value_parser = parse_hex_color)]
        accent: Option<String>,

        /// Primary type label, e.g. "Web App"
        #[arg(long = "project-type")]
        project_type: Option<String>,

        /// Path to a JSON file holding the terminal-cast transcript
        #[arg(long)]
        terminal_cast: Option<String>,

        /// Related project slugs/ids, comma-separated
        #[arg(long)]
        related: Option<String>,
    },

    /// Update an existing project
    Update {
        /// Project slug or UUID
        #[arg(name = "ref")]
        reference: String,

        /// Project name
        #[arg(short = 'n', long)]
        name: Option<String>,

        /// URL slug
        #[arg(long)]
        slug: Option<String>,

        /// Short description
        #[arg(short = 's', long)]
        short_desc: Option<String>,

        /// Project status
        #[arg(long, value_enum)]
        status: Option<StatusArg>,

        /// GitHub repository (use "" to clear)
        #[arg(long)]
        github_repo: Option<String>,

        /// Demo URL (use "" to clear)
        #[arg(long)]
        demo_url: Option<String>,

        /// Tag changes: +tag to add, -tag to remove (comma-separated)
        #[arg(short = 't', long)]
        tags: Option<String>,

        /// Accent color hex, e.g. "6366F1" or "#6366F1" (use "" to clear)
        #[arg(long, value_parser = parse_hex_color)]
        accent: Option<String>,

        /// Primary type label, e.g. "Web App" (use "" to clear)
        #[arg(long = "project-type")]
        project_type: Option<String>,

        /// Path to a JSON file with the terminal-cast transcript ("" to clear)
        #[arg(long)]
        terminal_cast: Option<String>,

        /// Related project slugs/ids, comma-separated; full replace ("" to clear)
        #[arg(long)]
        related: Option<String>,
    },

    /// Delete a project
    Delete {
        /// Project slug or UUID
        #[arg(name = "ref")]
        reference: String,
    },

    /// Edit detail-page block content
    #[command(subcommand)]
    Content(ProjectContentCommand),
}

#[derive(Subcommand, Debug)]
pub enum ProjectContentCommand {
    /// List blocks (id, type, preview)
    List {
        /// Project slug or UUID
        #[arg(name = "ref")]
        reference: String,
    },

    /// Print the document as JSON, or a single block by locator
    Get {
        /// Project slug or UUID
        #[arg(name = "ref")]
        reference: String,
        /// Block locator: a path like .3 or .3.0, or a block id; omit for the whole document
        locator: Option<String>,
    },

    /// Insert new block(s), authored as Markdown (--md) or a raw node (--node)
    Insert {
        /// Project slug or UUID
        #[arg(name = "ref")]
        reference: String,
        /// Position: start | end | before:<loc> | after:<loc> | prepend:<loc> | append:<loc>
        #[arg(long, default_value = "end")]
        at: String,
        /// Block(s) as Markdown, e.g. '## Heading' or '- a bullet'
        #[arg(long, conflicts_with = "node", allow_hyphen_values = true)]
        md: Option<String>,
        /// Raw ProseMirror node JSON (escape hatch), e.g. '{"type":"paragraph",...}'
        #[arg(long, required_unless_present = "md")]
        node: Option<String>,
        /// Print only the confirmation line, not the re-rendered document
        #[arg(long, short = 'q')]
        quiet: bool,
    },

    /// Replace a block, authored as Markdown (--md) or a raw node (--node)
    Replace {
        /// Project slug or UUID
        #[arg(name = "ref")]
        reference: String,
        /// Block locator: a path like .3 or .3.0, or a block id
        locator: String,
        /// Replacement as Markdown; multiple blocks replace the target then follow it
        #[arg(long, conflicts_with = "node", allow_hyphen_values = true)]
        md: Option<String>,
        /// Raw ProseMirror node JSON (escape hatch), keeps the target's position and id
        #[arg(long, required_unless_present = "md")]
        node: Option<String>,
        /// Print only the confirmation line, not the re-rendered document
        #[arg(long, short = 'q')]
        quiet: bool,
    },

    /// Remove a block
    Rm {
        /// Project slug or UUID
        #[arg(name = "ref")]
        reference: String,
        /// Block locator: a path like .3 or .3.0, or a block id
        locator: String,
        /// Print only the confirmation line, not the re-rendered document
        #[arg(long, short = 'q')]
        quiet: bool,
    },

    /// Move a block to a new position
    Move {
        /// Project slug or UUID
        #[arg(name = "ref")]
        reference: String,
        /// Block locator: a path like .3 or .3.0, or a block id
        locator: String,
        /// Position: start | end | before:<loc> | after:<loc> | prepend:<loc> | append:<loc>
        #[arg(long)]
        at: String,
        /// Print only the confirmation line, not the re-rendered document
        #[arg(long, short = 'q')]
        quiet: bool,
    },

    /// Replace the entire document from a JSON file
    Set {
        /// Project slug or UUID
        #[arg(name = "ref")]
        reference: String,
        /// Path to a file holding the full ProseMirror document as JSON
        #[arg(long)]
        file: String,
        /// Print only the confirmation line, not the re-rendered document
        #[arg(long, short = 'q')]
        quiet: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum TagsCommand {
    /// List all tags with project counts
    List,

    /// Get tag details with associated projects
    Get {
        /// Tag slug or UUID
        #[arg(name = "ref")]
        reference: String,
    },

    /// Create a new tag
    Create {
        /// Tag name
        name: String,

        /// URL slug (auto-generated from name if omitted)
        #[arg(long)]
        slug: Option<String>,

        /// Icon identifier (e.g., "simple-icons:rust")
        #[arg(long)]
        icon: Option<String>,

        /// Color hex, e.g. "3b82f6" or "#3b82f6"
        #[arg(long, value_parser = parse_hex_color)]
        color: Option<String>,
    },

    /// Update an existing tag
    Update {
        /// Tag slug or UUID
        #[arg(name = "ref")]
        reference: String,

        /// Tag name
        #[arg(short = 'n', long)]
        name: Option<String>,

        /// New URL slug
        #[arg(long)]
        slug: Option<String>,

        /// Icon identifier (use "" to clear)
        #[arg(long)]
        icon: Option<String>,

        /// Color hex, e.g. "3b82f6" or "#3b82f6" (use "" to clear)
        #[arg(long, value_parser = parse_hex_color)]
        color: Option<String>,
    },

    /// Delete a tag
    Delete {
        /// Tag slug or UUID
        #[arg(name = "ref")]
        reference: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum SettingsCommand {
    /// Get current site settings
    Get,

    /// Update site settings
    Update {
        /// Display name
        #[arg(long)]
        display_name: Option<String>,

        /// Occupation/title
        #[arg(long)]
        occupation: Option<String>,

        /// Bio text
        #[arg(long)]
        bio: Option<String>,

        /// Site title
        #[arg(long)]
        site_title: Option<String>,
    },
}

/// Tag operation for updates
#[derive(Debug, Clone)]
pub enum TagOp {
    Add(String),
    Remove(String),
}

/// Parse tags for create command (+ prefix optional)
pub fn parse_create_tags(s: &str) -> Vec<String> {
    s.split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(|t| t.strip_prefix('+').unwrap_or(t).to_string())
        .collect()
}

/// Parse tags for update command (+ or - prefix required)
pub fn parse_update_tags(s: &str) -> Result<Vec<TagOp>, String> {
    s.split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(|t| {
            if let Some(tag) = t.strip_prefix('+') {
                if tag.is_empty() {
                    Err("Tag name cannot be empty after '+'".to_string())
                } else {
                    Ok(TagOp::Add(tag.to_string()))
                }
            } else if let Some(tag) = t.strip_prefix('-') {
                if tag.is_empty() {
                    Err("Tag name cannot be empty after '-'".to_string())
                } else {
                    Ok(TagOp::Remove(tag.to_string()))
                }
            } else {
                Err(format!(
                    "Tag '{t}' requires prefix: use +{t} to add, -{t} to remove"
                ))
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse_hex_color;

    #[test]
    fn hex_color_accepts_bare_and_prefixed_and_normalizes() {
        assert_eq!(parse_hex_color("3B82F6").unwrap(), "3b82f6");
        assert_eq!(parse_hex_color("#3b82f6").unwrap(), "3b82f6");
        // Empty is the "clear this field" sentinel and passes through.
        assert_eq!(parse_hex_color("").unwrap(), "");
    }

    #[test]
    fn hex_color_rejects_bad_input() {
        assert!(parse_hex_color("xyz").is_err()); // non-hex
        assert!(parse_hex_color("abc").is_err()); // too short
        assert!(parse_hex_color("abcdef0").is_err()); // too long
        assert!(parse_hex_color("#12345g").is_err()); // non-hex with prefix
    }
}
