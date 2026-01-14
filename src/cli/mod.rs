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
    Api(ApiArgs),
}

#[derive(Parser, Debug)]
pub struct ApiArgs {
    /// API base URL
    #[arg(long, env = "API_BASE_URL", default_value = "http://localhost:8080")]
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
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .map(|t| t.strip_prefix('+').unwrap_or(t).to_string())
        .collect()
}

/// Parse tags for update command (+ or - prefix required)
pub fn parse_update_tags(s: &str) -> Result<Vec<TagOp>, String> {
    s.split(',')
        .map(|t| t.trim())
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
                    "Tag '{}' requires prefix: use +{} to add, -{} to remove",
                    t, t, t
                ))
            }
        })
        .collect()
}
