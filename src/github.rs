//! GitHub API client and activity sync scheduler.
//!
//! Implements a per-project scheduler that dynamically adjusts check intervals
//! based on activity recency. Projects with recent activity are checked more
//! frequently (down to 15 minutes), while stale projects are checked less often
//! (up to 24 hours).
//!
//! Fetches the latest activity from GitHub for projects that have `github_repo` set.
//! Only considers:
//! - Project-wide activity: Issues, PRs
//! - Main branch activity: Commits/pushes to the default branch only

use dashmap::DashMap;
use parking_lot::Mutex;
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;
use sqlx::PgPool;
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering as AtomicOrdering};
use std::time::{Duration, Instant};
use time::OffsetDateTime;
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::db::projects::DbProject;

static GITHUB_CLIENT: OnceCell<Option<Arc<GitHubClient>>> = OnceCell::const_new();

// Interval bounds (configurable via environment variables)
fn min_interval() -> Duration {
    let secs: u64 = std::env::var("GITHUB_SYNC_MIN_INTERVAL_SEC")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(15 * 60); // 15 minutes default
    Duration::from_secs(secs)
}

fn max_interval() -> Duration {
    let secs: u64 = std::env::var("GITHUB_SYNC_MAX_INTERVAL_SEC")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(24 * 60 * 60); // 24 hours default
    Duration::from_secs(secs)
}

/// Days of inactivity after which the maximum interval is used
const DAYS_TO_MAX: f64 = 90.0;

/// How often the scheduler checks for due items (hardcoded)
pub const SCHEDULER_TICK_INTERVAL: Duration = Duration::from_secs(30);

/// Calculate the check interval based on how recently the project had activity.
///
/// Uses a logarithmic curve that starts at MIN_INTERVAL for today's activity
/// and approaches MAX_INTERVAL as activity ages toward DAYS_TO_MAX.
///
/// Examples (with default 15min/24hr bounds):
/// - 0 days (today): 15 min
/// - 1 day: ~24 min
/// - 7 days: ~49 min
/// - 30 days: ~1.7 hr
/// - 90+ days: 24 hr
pub fn calculate_check_interval(last_activity: Option<OffsetDateTime>) -> Duration {
    let min = min_interval();
    let max = max_interval();

    let days_since = last_activity
        .map(|t| (OffsetDateTime::now_utc() - t).whole_days().max(0) as f64)
        .unwrap_or(DAYS_TO_MAX); // Default to max interval if no activity recorded

    // Logarithmic scaling: ln(1+days) / ln(1+90) gives 0..1 range
    let scale = (1.0 + days_since).ln() / (1.0 + DAYS_TO_MAX).ln();
    let scale = scale.clamp(0.0, 1.0);

    let interval_secs = min.as_secs_f64() + scale * (max.as_secs_f64() - min.as_secs_f64());

    Duration::from_secs(interval_secs as u64)
}

/// Statistics from scheduler operations
#[derive(Debug, Default)]
pub struct SyncStats {
    pub synced: u32,
    pub skipped: u32,
    pub errors: u32,
}

/// A project scheduled for GitHub activity checking
#[derive(Debug, Clone)]
pub struct ScheduledProject {
    /// When to next check this project
    pub next_check: Instant,
    /// Project database ID
    pub project_id: Uuid,
    /// GitHub repo in "owner/repo" format
    pub github_repo: String,
    /// Last known activity timestamp
    pub last_activity: Option<OffsetDateTime>,
    /// Consecutive error count (for exponential backoff)
    pub error_count: u32,
}

impl ScheduledProject {
    fn new(project: &DbProject, next_check: Instant) -> Option<Self> {
        Some(Self {
            next_check,
            project_id: project.id,
            github_repo: project.github_repo.clone()?,
            last_activity: project.last_github_activity,
            error_count: 0,
        })
    }

    /// Create a scheduled project for immediate checking
    pub fn immediate(project_id: Uuid, github_repo: String) -> Self {
        Self {
            next_check: Instant::now(),
            project_id,
            github_repo,
            last_activity: None,
            error_count: 0,
        }
    }
}

// Ordering for BinaryHeap (we use Reverse<> for min-heap behavior)
impl Eq for ScheduledProject {}

impl PartialEq for ScheduledProject {
    fn eq(&self, other: &Self) -> bool {
        self.project_id == other.project_id
    }
}

impl Ord for ScheduledProject {
    fn cmp(&self, other: &Self) -> Ordering {
        // Order by next_check time (earliest first when wrapped in Reverse)
        self.next_check.cmp(&other.next_check)
    }
}

impl PartialOrd for ScheduledProject {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// GitHub activity sync scheduler with priority queue
pub struct GitHubScheduler {
    /// Min-heap of scheduled projects (earliest check first)
    queue: Mutex<BinaryHeap<Reverse<ScheduledProject>>>,
    /// Running statistics
    pub total_synced: AtomicU32,
    pub total_skipped: AtomicU32,
    pub total_errors: AtomicU32,
}

impl GitHubScheduler {
    /// Create a new scheduler with projects from the database.
    ///
    /// Projects are sorted by activity recency and their initial checks are
    /// staggered across their calculated intervals to prevent clumping.
    pub fn new(mut projects: Vec<DbProject>) -> Self {
        // Sort by activity recency (most recent first = shortest intervals first)
        projects.sort_by(|a, b| {
            let a_time = a.last_github_activity.unwrap_or(a.created_at);
            let b_time = b.last_github_activity.unwrap_or(b.created_at);
            b_time.cmp(&a_time) // Descending (most recent first)
        });

        let now = Instant::now();
        let total = projects.len().max(1) as f64;

        let scheduled: Vec<_> = projects
            .into_iter()
            .enumerate()
            .filter_map(|(i, project)| {
                let interval = calculate_check_interval(project.last_github_activity);

                // Stagger initial checks: position i of n gets (i/n * interval) offset
                // This spreads checks evenly across each project's interval
                let offset_secs = (i as f64 / total) * interval.as_secs_f64();
                let next_check = now + Duration::from_secs_f64(offset_secs);

                ScheduledProject::new(&project, next_check)
            })
            .map(Reverse)
            .collect();

        let queue = BinaryHeap::from(scheduled);

        Self {
            queue: Mutex::new(queue),
            total_synced: AtomicU32::new(0),
            total_skipped: AtomicU32::new(0),
            total_errors: AtomicU32::new(0),
        }
    }

    /// Number of projects in the scheduler
    pub fn len(&self) -> usize {
        self.queue.lock().len()
    }

    /// Check if the scheduler has no projects
    pub fn is_empty(&self) -> bool {
        self.queue.lock().is_empty()
    }

    /// Pop the next due project, if any.
    ///
    /// Returns None if the queue is empty or the next project isn't due yet.
    pub fn pop_if_due(&self) -> Option<ScheduledProject> {
        let mut queue = self.queue.lock();
        let now = Instant::now();

        // Peek to check if the next item is due
        if queue.peek().is_some_and(|p| p.0.next_check <= now) {
            queue.pop().map(|r| r.0)
        } else {
            None
        }
    }

    /// Reschedule a project after a successful sync.
    pub fn reschedule(&self, mut project: ScheduledProject, new_activity: Option<OffsetDateTime>) {
        project.last_activity = new_activity.or(project.last_activity);
        project.error_count = 0; // Reset error count on success

        let interval = calculate_check_interval(project.last_activity);
        project.next_check = Instant::now() + interval;

        self.queue.lock().push(Reverse(project));
        self.total_synced.fetch_add(1, AtomicOrdering::Relaxed);
    }

    /// Reschedule a project after an error with exponential backoff.
    ///
    /// Each consecutive error doubles the wait time, capped at max_interval.
    pub fn reschedule_with_error(&self, mut project: ScheduledProject) {
        project.error_count += 1;

        // Exponential backoff: base_interval * 2^error_count, capped at max
        let base = calculate_check_interval(project.last_activity);
        let multiplier = 2u32.saturating_pow(project.error_count.min(6)); // Cap at 2^6 = 64x
        let backoff = base.saturating_mul(multiplier);
        let interval = backoff.min(max_interval());

        project.next_check = Instant::now() + interval;

        tracing::debug!(
            repo = %project.github_repo,
            error_count = project.error_count,
            backoff_mins = interval.as_secs() / 60,
            "Rescheduled with error backoff"
        );

        self.queue.lock().push(Reverse(project));
        self.total_errors.fetch_add(1, AtomicOrdering::Relaxed);
    }

    /// Mark a project as skipped (no update needed).
    pub fn reschedule_skipped(&self, mut project: ScheduledProject) {
        project.error_count = 0; // Reset on successful check (even if skipped)

        let interval = calculate_check_interval(project.last_activity);
        project.next_check = Instant::now() + interval;

        self.queue.lock().push(Reverse(project));
        self.total_skipped.fetch_add(1, AtomicOrdering::Relaxed);
    }

    /// Back off all projects by a fixed duration (e.g., after rate limiting).
    pub fn backoff_all(&self, backoff: Duration) {
        let mut queue = self.queue.lock();
        let now = Instant::now();

        // Drain and re-add with updated times
        let projects: Vec<_> = queue.drain().collect();
        for mut p in projects {
            // Only push back projects that aren't already further out
            if p.0.next_check < now + backoff {
                p.0.next_check = now + backoff;
            }
            queue.push(p);
        }
    }

    /// Add a new project to the scheduler for immediate checking.
    pub fn add_project(&self, project_id: Uuid, github_repo: String) {
        let project = ScheduledProject::immediate(project_id, github_repo);
        self.queue.lock().push(Reverse(project));
    }

    /// Remove a project from the scheduler.
    pub fn remove_project(&self, project_id: Uuid) {
        let mut queue = self.queue.lock();
        let projects: Vec<_> = queue
            .drain()
            .filter(|p| p.0.project_id != project_id)
            .collect();
        queue.extend(projects);
    }

    /// Get current statistics snapshot
    pub fn stats(&self) -> SyncStats {
        SyncStats {
            synced: self.total_synced.load(AtomicOrdering::Relaxed),
            skipped: self.total_skipped.load(AtomicOrdering::Relaxed),
            errors: self.total_errors.load(AtomicOrdering::Relaxed),
        }
    }

    /// Get info about the next scheduled check (for logging/debugging)
    pub fn next_check_info(&self) -> Option<(String, Duration)> {
        let queue = self.queue.lock();
        queue.peek().map(|p| {
            let until = p.0.next_check.saturating_duration_since(Instant::now());
            (p.0.github_repo.clone(), until)
        })
    }
}

/// Global scheduler instance (initialized during server startup)
static GITHUB_SCHEDULER: OnceCell<Arc<GitHubScheduler>> = OnceCell::const_new();

/// Get the global scheduler instance
pub fn get_scheduler() -> Option<Arc<GitHubScheduler>> {
    GITHUB_SCHEDULER.get().cloned()
}

/// Initialize the global scheduler with projects from the database
pub async fn init_scheduler(pool: &PgPool) -> Option<Arc<GitHubScheduler>> {
    // Only init if client is available
    GitHubClient::get().await.as_ref()?;

    let projects = crate::db::projects::get_projects_with_github_repo(pool)
        .await
        .ok()?;

    let scheduler = Arc::new(GitHubScheduler::new(projects));

    // Store globally
    let _ = GITHUB_SCHEDULER.set(scheduler.clone());

    Some(scheduler)
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

/// Sync a single project's GitHub activity.
///
/// Returns the new activity timestamp if found, or None if no activity.
pub async fn sync_single_project(
    client: &GitHubClient,
    pool: &PgPool,
    project: &ScheduledProject,
) -> Result<Option<OffsetDateTime>, GitHubError> {
    let (owner, repo) = parse_github_repo(&project.github_repo)
        .ok_or_else(|| GitHubError::Api(400, "Invalid github_repo format".to_string()))?;

    let activity_time = client.get_latest_activity(owner, repo).await?;

    if let Some(new_time) = activity_time {
        // Only update if newer than current value
        let should_update = project
            .last_activity
            .is_none_or(|current| new_time > current);

        if should_update {
            crate::db::projects::update_last_github_activity(pool, project.project_id, new_time)
                .await
                .map_err(|e| GitHubError::Api(500, e.to_string()))?;

            tracing::debug!(
                repo = %project.github_repo,
                activity_time = %new_time,
                "Updated last_github_activity"
            );
        }
    }

    Ok(activity_time)
}

/// Run the GitHub activity sync scheduler loop.
///
/// This is the main entry point for the background sync task.
/// It checks for due projects every 30 seconds and syncs them individually.
pub async fn run_scheduler(pool: PgPool) {
    let client = match GitHubClient::get().await {
        Some(c) => c,
        None => return, // GitHub sync disabled
    };

    let scheduler = match init_scheduler(&pool).await {
        Some(s) => s,
        None => {
            tracing::warn!("Failed to initialize GitHub scheduler");
            return;
        }
    };

    if scheduler.is_empty() {
        tracing::info!("GitHub scheduler: no projects with github_repo configured");
        return;
    }

    // Log initial state
    if let Some((next_repo, until)) = scheduler.next_check_info() {
        tracing::info!(
            projects = scheduler.len(),
            next_repo = %next_repo,
            next_check_secs = until.as_secs(),
            "GitHub scheduler started with staggered checks"
        );
    } else {
        tracing::info!(projects = scheduler.len(), "GitHub scheduler started");
    }

    // Check for due projects every 30 seconds
    let mut tick = tokio::time::interval(SCHEDULER_TICK_INTERVAL);

    loop {
        tick.tick().await;

        // Process all due projects
        let mut processed = 0u32;
        while let Some(project) = scheduler.pop_if_due() {
            processed += 1;

            match sync_single_project(&client, &pool, &project).await {
                Ok(new_activity) => {
                    let interval = calculate_check_interval(new_activity);
                    tracing::debug!(
                        repo = %project.github_repo,
                        next_check_mins = interval.as_secs() / 60,
                        "GitHub sync complete"
                    );

                    // Check if activity changed
                    if new_activity != project.last_activity {
                        scheduler.reschedule(project, new_activity);
                    } else {
                        scheduler.reschedule_skipped(project);
                    }
                }
                Err(GitHubError::RateLimited) => {
                    // Back off ALL projects by 5 minutes
                    scheduler.backoff_all(Duration::from_secs(5 * 60));
                    tracing::warn!(
                        processed,
                        "GitHub rate limit hit, backing off all projects 5 minutes"
                    );
                    // Re-add the current project too
                    scheduler.reschedule_with_error(project);
                    break;
                }
                Err(GitHubError::NotFound(repo)) => {
                    // Repo doesn't exist or is private - use long interval
                    tracing::warn!(
                        repo = %repo,
                        "GitHub repo not found or inaccessible, using max interval"
                    );
                    scheduler.reschedule_skipped(project);
                }
                Err(e) => {
                    tracing::warn!(
                        repo = %project.github_repo,
                        error = %e,
                        "GitHub sync error, scheduling retry with backoff"
                    );
                    scheduler.reschedule_with_error(project);
                }
            }
        }

        // Log periodic stats (every ~5 minutes worth of ticks = 10 ticks)
        static TICK_COUNT: AtomicU32 = AtomicU32::new(0);
        let ticks = TICK_COUNT.fetch_add(1, AtomicOrdering::Relaxed);
        if ticks > 0 && ticks.is_multiple_of(10) {
            let stats = scheduler.stats();
            if let Some((next_repo, until)) = scheduler.next_check_info() {
                tracing::info!(
                    synced = stats.synced,
                    skipped = stats.skipped,
                    errors = stats.errors,
                    next_repo = %next_repo,
                    next_check_secs = until.as_secs(),
                    "GitHub scheduler stats"
                );
            }
        }
    }
}
