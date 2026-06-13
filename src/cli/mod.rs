pub mod api;
pub mod client;
pub mod output;
pub mod seed;
pub mod serve;

use clap::{Parser, Subcommand};

use crate::config::ListenAddr;

/// xevion.dev - Personal portfolio server and API client
#[derive(Parser, Debug)]
#[command(name = "xevion")]
#[command(about = "Personal portfolio server with API client for content management")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Address(es) to listen on (TCP or Unix socket)
    #[arg(long, env = "LISTEN_ADDR", value_delimiter = ',')]
    pub listen: Vec<ListenAddr>,

    /// Downstream SSR server URL
    #[arg(long, env = "DOWNSTREAM_URL")]
    pub downstream: Option<String>,

    /// Trust X-Request-ID header from specified source
    #[arg(long, env = "TRUST_REQUEST_ID")]
    pub trust_request_id: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Seed the database with sample data
    Seed,

    /// API client for managing content remotely
    Api(Box<ApiArgs>),
}

#[derive(Parser, Debug)]
pub struct ApiArgs {
    /// API base URL
    #[arg(long, env = "API_BASE_URL", default_value = "http://localhost:10237")]
    pub api_url: String,

    /// Session file path
    #[arg(long, env = "XEVION_SESSION", default_value = ".xevion-session")]
    pub session: String,

    /// Output raw JSON instead of formatted text
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: ApiCommand,
}

#[derive(Subcommand, Debug)]
pub enum ApiCommand {
    /// Login and save session
    Login {
        /// Username
        username: String,
        /// Password
        password: String,
    },

    /// Clear saved session
    Logout,

    /// Check current session status
    Session,

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

        /// Full description
        #[arg(short = 'd', long)]
        desc: String,

        /// URL slug (auto-generated from name if omitted)
        #[arg(long)]
        slug: Option<String>,

        /// Project status
        #[arg(long, default_value = "active")]
        status: String,

        /// GitHub repository (e.g., "Xevion/xevion.dev")
        #[arg(long)]
        github_repo: Option<String>,

        /// Demo URL
        #[arg(long)]
        demo_url: Option<String>,

        /// Tags to add (comma-separated, + prefix optional)
        #[arg(short = 't', long)]
        tags: Option<String>,
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

        /// Full description
        #[arg(short = 'd', long)]
        desc: Option<String>,

        /// Project status
        #[arg(long)]
        status: Option<String>,

        /// GitHub repository (use "" to clear)
        #[arg(long)]
        github_repo: Option<String>,

        /// Demo URL (use "" to clear)
        #[arg(long)]
        demo_url: Option<String>,

        /// Tag changes: +tag to add, -tag to remove (comma-separated)
        #[arg(short = 't', long)]
        tags: Option<String>,
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

/// One source of a block's `data`. Exactly one must be given.
#[derive(clap::Args, Debug)]
pub struct BlockContentArgs {
    /// Markdown body for a prose block (builds `{"md": ...}`)
    #[arg(long)]
    pub md: Option<String>,

    /// Read the markdown body from a file
    #[arg(long)]
    pub file: Option<String>,

    /// Raw JSON for the block's `data` field (any block type)
    #[arg(long)]
    pub data: Option<String>,

    /// Read raw JSON `data` from a file
    #[arg(long = "data-file")]
    pub data_file: Option<String>,
}

impl BlockContentArgs {
    /// Build a block's `data` from exactly one content source.
    /// Returns `(data, is_prose_shorthand)`.
    pub fn build_data(&self) -> Result<(serde_json::Value, bool), Box<dyn std::error::Error>> {
        let count = [
            self.md.is_some(),
            self.file.is_some(),
            self.data.is_some(),
            self.data_file.is_some(),
        ]
        .into_iter()
        .filter(|set| *set)
        .count();
        if count == 0 {
            return Err("provide one of --md, --file, --data, or --data-file".into());
        }
        if count > 1 {
            return Err("provide only one of --md, --file, --data, --data-file".into());
        }

        if let Some(text) = &self.md {
            Ok((serde_json::json!({ "md": text }), true))
        } else if let Some(path) = &self.file {
            Ok((
                serde_json::json!({ "md": std::fs::read_to_string(path)? }),
                true,
            ))
        } else if let Some(raw) = &self.data {
            Ok((serde_json::from_str(raw)?, false))
        } else {
            let raw = std::fs::read_to_string(self.data_file.as_ref().unwrap())?;
            Ok((serde_json::from_str(&raw)?, false))
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum ProjectContentCommand {
    /// List blocks (id, type, preview)
    List {
        /// Project slug or UUID
        #[arg(name = "ref")]
        reference: String,
    },

    /// Print the document as JSON, or a single block by id
    Get {
        /// Project slug or UUID
        #[arg(name = "ref")]
        reference: String,
        /// Block id; omit for the whole document
        block_id: Option<String>,
    },

    /// Insert a new block
    Insert {
        /// Project slug or UUID
        #[arg(name = "ref")]
        reference: String,
        /// Block type (defaults to "prose" with --md/--file)
        #[arg(long = "type")]
        r#type: Option<String>,
        /// Position: start | end | after:<id> | before:<id>
        #[arg(long, default_value = "end")]
        at: String,
        #[command(flatten)]
        content: BlockContentArgs,
    },

    /// Replace a block's data (and optionally its type)
    Set {
        /// Project slug or UUID
        #[arg(name = "ref")]
        reference: String,
        /// Block id
        block_id: String,
        /// New block type (omit to keep the existing one)
        #[arg(long = "type")]
        r#type: Option<String>,
        #[command(flatten)]
        content: BlockContentArgs,
    },

    /// Remove a block
    Rm {
        /// Project slug or UUID
        #[arg(name = "ref")]
        reference: String,
        /// Block id
        block_id: String,
    },

    /// Move a block to a new position
    Move {
        /// Project slug or UUID
        #[arg(name = "ref")]
        reference: String,
        /// Block id
        block_id: String,
        /// Position: start | end | after:<id> | before:<id>
        #[arg(long)]
        at: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum TagsCommand {
    /// List all tags with project counts
    List,

    /// Get tag details with associated projects
    Get {
        /// Tag slug
        slug: String,
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

        /// Color hex without # (e.g., "3b82f6")
        #[arg(long)]
        color: Option<String>,
    },

    /// Update an existing tag
    Update {
        /// Tag slug
        slug: String,

        /// Tag name
        #[arg(short = 'n', long)]
        name: Option<String>,

        /// New URL slug
        #[arg(long = "new-slug")]
        new_slug: Option<String>,

        /// Icon identifier (use "" to clear)
        #[arg(long)]
        icon: Option<String>,

        /// Color hex (use "" to clear)
        #[arg(long)]
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
