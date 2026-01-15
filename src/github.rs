//! GitHub API client for syncing repository activity.
//!
//! Fetches the latest activity from GitHub for projects that have `github_repo` set.
//! Only considers:
//! - Project-wide activity: Issues, PRs
//! - Main branch activity: Commits/pushes to the default branch only

use dashmap::DashMap;
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use time::OffsetDateTime;
use tokio::sync::OnceCell;

static GITHUB_CLIENT: OnceCell<Option<Arc<GitHubClient>>> = OnceCell::const_new();

/// Statistics from a sync run
#[derive(Debug, Default)]
pub struct SyncStats {
    pub synced: u32,
    pub skipped: u32,
    pub errors: u32,
}

/// GitHub API client with caching for default branches
pub struct GitHubClient {
    client: reqwest::Client,
    /// Cache of "owner/repo" -> default_branch
    branch_cache: DashMap<String, String>,
}

// GitHub API response types

#[derive(Debug, Deserialize)]
struct RepoInfo {
    default_branch: String,
}

#[derive(Debug, Deserialize)]
struct ActivityEvent {
    /// The activity endpoint uses `timestamp`, not `created_at`
    timestamp: String,
}

#[derive(Debug, Deserialize)]
struct IssueEvent {
    created_at: String,
}

/// Errors that can occur during GitHub API operations
#[derive(Debug)]
pub enum GitHubError {
    /// HTTP request failed
    Request(reqwest::Error),
    /// Repository not found (404)
    NotFound(String),
    /// Rate limited (429)
    RateLimited,
    /// Failed to parse timestamp
    ParseTime(String),
    /// Other API error
    Api(u16, String),
}

impl std::fmt::Display for GitHubError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitHubError::Request(e) => write!(f, "HTTP request failed: {e}"),
            GitHubError::NotFound(repo) => write!(f, "Repository not found: {repo}"),
            GitHubError::RateLimited => write!(f, "GitHub API rate limit exceeded"),
            GitHubError::ParseTime(s) => write!(f, "Failed to parse timestamp: {s}"),
            GitHubError::Api(status, msg) => write!(f, "GitHub API error ({status}): {msg}"),
        }
    }
}

impl std::error::Error for GitHubError {}

impl GitHubClient {
    /// Create a new GitHub client if GITHUB_TOKEN is set.
    fn new() -> Option<Self> {
        let token = std::env::var("GITHUB_TOKEN").ok()?;

        if token.is_empty() {
            return None;
        }

        let mut headers = HeaderMap::new();
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );
        headers.insert(
            "X-GitHub-Api-Version",
            HeaderValue::from_static("2022-11-28"),
        );
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}")).ok()?,
        );
        headers.insert(USER_AGENT, HeaderValue::from_static("xevion-dev/1.0"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .ok()?;

        Some(Self {
            client,
            branch_cache: DashMap::new(),
        })
    }

    /// Get the shared GitHub client instance.
    /// Returns None if GITHUB_TOKEN is not set, logging a warning once.
    pub async fn get() -> Option<Arc<Self>> {
        GITHUB_CLIENT
            .get_or_init(|| async {
                match GitHubClient::new() {
                    Some(client) => {
                        tracing::info!("GitHub sync client initialized");
                        Some(Arc::new(client))
                    }
                    None => {
                        tracing::warn!(
                            "GitHub sync disabled: GITHUB_TOKEN not set. \
                             Set GITHUB_TOKEN to enable automatic activity sync."
                        );
                        None
                    }
                }
            })
            .await
            .clone()
    }

    /// Fetch repository info (primarily for default_branch).
    async fn get_repo_info(&self, owner: &str, repo: &str) -> Result<RepoInfo, GitHubError> {
        let url = format!("https://api.github.com/repos/{owner}/{repo}");

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(GitHubError::Request)?;

        let status = response.status();

        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(GitHubError::NotFound(format!("{owner}/{repo}")));
        }

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(GitHubError::RateLimited);
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(GitHubError::Api(status.as_u16(), body));
        }

        response
            .json::<RepoInfo>()
            .await
            .map_err(GitHubError::Request)
    }

    /// Get the default branch for a repo, using cache if available.
    async fn get_default_branch(&self, owner: &str, repo: &str) -> Result<String, GitHubError> {
        let cache_key = format!("{owner}/{repo}");

        // Check cache first
        if let Some(branch) = self.branch_cache.get(&cache_key) {
            return Ok(branch.clone());
        }

        // Fetch from API
        let info = self.get_repo_info(owner, repo).await?;
        self.branch_cache
            .insert(cache_key, info.default_branch.clone());
        Ok(info.default_branch)
    }

    /// Fetch the latest activity on the default branch.
    /// Returns the timestamp of the most recent push, if any.
    async fn get_latest_branch_activity(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
    ) -> Result<Option<OffsetDateTime>, GitHubError> {
        let url =
            format!("https://api.github.com/repos/{owner}/{repo}/activity?ref={branch}&per_page=1");

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(GitHubError::Request)?;

        let status = response.status();

        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(GitHubError::NotFound(format!("{owner}/{repo}")));
        }

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(GitHubError::RateLimited);
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(GitHubError::Api(status.as_u16(), body));
        }

        let events: Vec<ActivityEvent> = response.json().await.map_err(GitHubError::Request)?;

        if let Some(event) = events.first() {
            let timestamp = OffsetDateTime::parse(
                &event.timestamp,
                &time::format_description::well_known::Rfc3339,
            )
            .map_err(|_| GitHubError::ParseTime(event.timestamp.clone()))?;
            Ok(Some(timestamp))
        } else {
            Ok(None)
        }
    }

    /// Fetch the latest issue/PR event.
    /// Returns the timestamp of the most recent event, if any.
    async fn get_latest_issue_event(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Option<OffsetDateTime>, GitHubError> {
        let url = format!("https://api.github.com/repos/{owner}/{repo}/issues/events?per_page=1");

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(GitHubError::Request)?;

        let status = response.status();

        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(GitHubError::NotFound(format!("{owner}/{repo}")));
        }

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(GitHubError::RateLimited);
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(GitHubError::Api(status.as_u16(), body));
        }

        let events: Vec<IssueEvent> = response.json().await.map_err(GitHubError::Request)?;

        if let Some(event) = events.first() {
            let timestamp = OffsetDateTime::parse(
                &event.created_at,
                &time::format_description::well_known::Rfc3339,
            )
            .map_err(|_| GitHubError::ParseTime(event.created_at.clone()))?;
            Ok(Some(timestamp))
        } else {
            Ok(None)
        }
    }

    /// Fetch the latest activity for a repository.
    /// Considers both branch activity and issue/PR events, returning the most recent.
    pub async fn get_latest_activity(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Option<OffsetDateTime>, GitHubError> {
        // Get default branch (cached)
        let branch = self.get_default_branch(owner, repo).await?;

        // Fetch both activity sources in parallel
        let (branch_activity, issue_activity) = tokio::join!(
            self.get_latest_branch_activity(owner, repo, &branch),
            self.get_latest_issue_event(owner, repo)
        );

        // Take the most recent timestamp from either source
        let branch_time = branch_activity?;
        let issue_time = issue_activity?;

        Ok(match (branch_time, issue_time) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        })
    }
}

/// Parse a "owner/repo" string into (owner, repo) tuple.
fn parse_github_repo(github_repo: &str) -> Option<(&str, &str)> {
    let parts: Vec<&str> = github_repo.split('/').collect();
    if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
        Some((parts[0], parts[1]))
    } else {
        None
    }
}

/// Sync GitHub activity for all projects that have github_repo set.
/// Updates the last_github_activity field with the most recent activity timestamp.
pub async fn sync_github_activity(pool: &PgPool) -> Result<SyncStats, Box<dyn std::error::Error>> {
    let client = GitHubClient::get()
        .await
        .ok_or("GitHub client not initialized")?;

    let projects = crate::db::projects::get_projects_with_github_repo(pool).await?;
    let mut stats = SyncStats::default();

    tracing::debug!(count = projects.len(), "Starting GitHub activity sync");

    for project in projects {
        let github_repo = match &project.github_repo {
            Some(repo) => repo,
            None => continue,
        };

        let (owner, repo) = match parse_github_repo(github_repo) {
            Some(parsed) => parsed,
            None => {
                tracing::warn!(
                    project_id = %project.id,
                    github_repo = %github_repo,
                    "Invalid github_repo format, expected 'owner/repo'"
                );
                stats.skipped += 1;
                continue;
            }
        };

        match client.get_latest_activity(owner, repo).await {
            Ok(Some(activity_time)) => {
                // Only update if newer than current value
                let should_update = project
                    .last_github_activity
                    .is_none_or(|current| activity_time > current);

                if should_update {
                    if let Err(e) = crate::db::projects::update_last_github_activity(
                        pool,
                        project.id,
                        activity_time,
                    )
                    .await
                    {
                        tracing::error!(
                            project_id = %project.id,
                            error = %e,
                            "Failed to update last_github_activity"
                        );
                        stats.errors += 1;
                    } else {
                        tracing::debug!(
                            project_id = %project.id,
                            github_repo = %github_repo,
                            activity_time = %activity_time,
                            "Updated last_github_activity"
                        );
                        stats.synced += 1;
                    }
                } else {
                    stats.skipped += 1;
                }
            }
            Ok(None) => {
                tracing::debug!(
                    project_id = %project.id,
                    github_repo = %github_repo,
                    "No activity found for repository"
                );
                stats.skipped += 1;
            }
            Err(GitHubError::NotFound(repo)) => {
                tracing::warn!(
                    project_id = %project.id,
                    github_repo = %repo,
                    "Repository not found or inaccessible, skipping"
                );
                stats.skipped += 1;
            }
            Err(GitHubError::RateLimited) => {
                tracing::warn!("GitHub API rate limit hit, stopping sync early");
                stats.errors += 1;
                break;
            }
            Err(e) => {
                tracing::error!(
                    project_id = %project.id,
                    github_repo = %github_repo,
                    error = %e,
                    "Failed to fetch GitHub activity"
                );
                stats.errors += 1;
            }
        }
    }

    Ok(stats)
}
