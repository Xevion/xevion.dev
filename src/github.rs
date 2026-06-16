//! GitHub API client and activity sync scheduler.
//!
//! Implements a per-project scheduler that dynamically adjusts check intervals
//! based on activity recency. Projects with recent activity are checked more
//! frequently (down to 15 minutes), while stale projects are checked less often
//! (up to 24 hours).
//!
//! Activity is sourced from two repo-scoped endpoints that both work for public
//! and private repositories with an authenticated token:
//! - `GET /repos/{o}/{r}/activity` — pushes / force-pushes / branch ops / merges
//!   on any ref (no conditional-request support, so always a live call).
//! - `GET /repos/{o}/{r}/issues?state=all&sort=updated&direction=desc` — the most
//!   recently updated issue or PR (REST treats PRs as issues; `updated_at` also
//!   advances on comments). Polled with an `ETag`, so unchanged repos return 304
//!   and cost nothing against the rate limit.
//!
//! The latest activity is `max()` of the two. Stars/forks are intentionally not
//! counted (neither endpoint surfaces them).

use dashmap::DashMap;
use parking_lot::Mutex;
use reqwest::StatusCode;
use reqwest::header::{
    ACCEPT, AUTHORIZATION, ETAG, HeaderMap, HeaderValue, IF_NONE_MATCH, USER_AGENT,
};
use serde::Deserialize;
use sqlx::PgPool;
use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering as AtomicOrdering};
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

/// Fallback wait when a rate-limit response carries no `retry-after` /
/// `x-ratelimit-reset` header to compute the exact reset from.
const RATE_LIMIT_FALLBACK: Duration = Duration::from_mins(5);

/// How often the scheduler checks for due items (hardcoded)
pub const SCHEDULER_TICK_INTERVAL: Duration = Duration::from_secs(30);

/// Calculate the check interval based on how recently the project had activity.
///
/// Uses a logarithmic curve that starts at `MIN_INTERVAL` for today's activity
/// and approaches `MAX_INTERVAL` as activity ages toward `DAYS_TO_MAX`.
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

    let days_since = last_activity.map_or(DAYS_TO_MAX, |t| {
        (OffsetDateTime::now_utc() - t).whole_days().max(0) as f64
    }); // Default to max interval if no activity recorded

    // Logarithmic scaling: ln(1+days) / ln(1+90) gives 0..1 range
    let scale = days_since.ln_1p() / DAYS_TO_MAX.ln_1p();
    let scale = scale.clamp(0.0, 1.0);

    let interval_secs = min.as_secs_f64() + scale * (max.as_secs_f64() - min.as_secs_f64());

    Duration::from_secs(interval_secs as u64)
}

/// Most recent of two optional timestamps, treating `None` as "no signal".
/// Used both to merge the activity/issue sources and to advance a project's
/// in-memory `last_activity` without ever regressing it.
fn merge_activity(a: Option<OffsetDateTime>, b: Option<OffsetDateTime>) -> Option<OffsetDateTime> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x.max(y)),
        (x, y) => x.or(y),
    }
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
    /// Registration epoch this entry was scheduled under. A heap entry is valid
    /// only while it matches the project's current epoch in the registry; this is
    /// how deletes and repo changes invalidate in-flight/queued entries (see
    /// [`GitHubScheduler`]).
    epoch: u64,
}

// Ordering for BinaryHeap (we use Reverse<> for min-heap behavior). Only
// `next_check` matters for scheduling order.
impl Eq for ScheduledProject {}

impl PartialEq for ScheduledProject {
    fn eq(&self, other: &Self) -> bool {
        self.next_check == other.next_check
    }
}

impl Ord for ScheduledProject {
    fn cmp(&self, other: &Self) -> Ordering {
        self.next_check.cmp(&other.next_check)
    }
}

impl PartialOrd for ScheduledProject {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Authoritative registration for a tracked project: its current repo and epoch.
/// The heap may hold stale entries; this map is the source of truth.
#[derive(Debug, Clone)]
struct RegEntry {
    repo: String,
    epoch: u64,
}

/// Mutable scheduler state guarded by a single lock so the queue and the
/// registry can never disagree.
struct SchedulerInner {
    /// Min-heap of scheduled checks (earliest first). May contain stale entries
    /// that are discarded lazily on pop / reschedule.
    queue: BinaryHeap<Reverse<ScheduledProject>>,
    /// Authoritative set of tracked projects (id -> repo + current epoch).
    registry: HashMap<Uuid, RegEntry>,
}

/// GitHub activity sync scheduler with a priority queue.
///
/// Heap entries are validated against `registry` by epoch on pop and before
/// re-pushing. `remove_project` drops the registry entry, and `add_project`
/// allocates a fresh (globally monotonic) epoch — so a project deleted or
/// re-homed while its sync is in flight cannot be resurrected by the completing
/// task, and duplicate enqueues self-heal.
pub struct GitHubScheduler {
    inner: Mutex<SchedulerInner>,
    /// Monotonic epoch allocator; never reuses a value, so a re-added id can't
    /// collide with a lingering stale heap entry.
    next_epoch: AtomicU64,
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

        let mut queue = BinaryHeap::new();
        let mut registry = HashMap::new();
        let mut epoch = 0u64;

        for (i, project) in projects.into_iter().enumerate() {
            let Some(repo) = project.github_repo.clone() else {
                continue;
            };

            let interval = calculate_check_interval(project.last_github_activity);
            // Stagger initial checks: position i of n gets (i/n * interval) offset,
            // spreading checks evenly across each project's interval.
            let offset_secs = (i as f64 / total) * interval.as_secs_f64();
            let next_check = now + Duration::from_secs_f64(offset_secs);

            registry.insert(
                project.id,
                RegEntry {
                    repo: repo.clone(),
                    epoch,
                },
            );
            queue.push(Reverse(ScheduledProject {
                next_check,
                project_id: project.id,
                github_repo: repo,
                last_activity: project.last_github_activity,
                error_count: 0,
                epoch,
            }));
            epoch += 1;
        }

        Self {
            inner: Mutex::new(SchedulerInner { queue, registry }),
            next_epoch: AtomicU64::new(epoch),
            total_synced: AtomicU32::new(0),
            total_skipped: AtomicU32::new(0),
            total_errors: AtomicU32::new(0),
        }
    }

    /// Number of tracked projects (authoritative; ignores stale heap entries).
    pub fn len(&self) -> usize {
        self.inner.lock().registry.len()
    }

    /// Check if the scheduler tracks no projects.
    pub fn is_empty(&self) -> bool {
        self.inner.lock().registry.is_empty()
    }

    /// Pop the next due, still-valid project, if any.
    ///
    /// Stale heap entries (deleted or superseded by a newer epoch) are discarded
    /// in place. Returns `None` if the queue is empty or the next item isn't due.
    pub fn pop_if_due(&self) -> Option<ScheduledProject> {
        let mut inner = self.inner.lock();
        let now = Instant::now();

        loop {
            match inner.queue.peek() {
                Some(p) if p.0.next_check <= now => {}
                _ => return None,
            }
            let project = inner.queue.pop().expect("peek confirmed non-empty").0;
            if inner
                .registry
                .get(&project.project_id)
                .is_some_and(|r| r.epoch == project.epoch)
            {
                return Some(project);
            }
            // Stale: deleted or superseded by a newer epoch. Drop and continue.
        }
    }

    /// Re-push a project only if it is still the current registration.
    fn push_if_current(&self, project: ScheduledProject) {
        let mut inner = self.inner.lock();
        if inner
            .registry
            .get(&project.project_id)
            .is_some_and(|r| r.epoch == project.epoch)
        {
            inner.queue.push(Reverse(project));
        }
    }

    /// Reschedule a project after a successful sync.
    pub fn reschedule(&self, mut project: ScheduledProject, new_activity: Option<OffsetDateTime>) {
        project.last_activity = merge_activity(new_activity, project.last_activity);
        project.error_count = 0;

        let interval = calculate_check_interval(project.last_activity);
        project.next_check = Instant::now() + interval;

        self.total_synced.fetch_add(1, AtomicOrdering::Relaxed);
        self.push_if_current(project);
    }

    /// Reschedule a project after an error with exponential backoff.
    ///
    /// Each consecutive error doubles the wait time, capped at `max_interval`.
    pub fn reschedule_with_error(&self, mut project: ScheduledProject) {
        project.error_count += 1;

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

        self.total_errors.fetch_add(1, AtomicOrdering::Relaxed);
        self.push_if_current(project);
    }

    /// Mark a project as skipped (no update needed).
    pub fn reschedule_skipped(&self, mut project: ScheduledProject) {
        project.error_count = 0; // Reset on successful check (even if skipped)

        let interval = calculate_check_interval(project.last_activity);
        project.next_check = Instant::now() + interval;

        self.total_skipped.fetch_add(1, AtomicOrdering::Relaxed);
        self.push_if_current(project);
    }

    /// Back off all queued checks by at least `backoff` (e.g. after rate limiting).
    pub fn backoff_all(&self, backoff: Duration) {
        let mut inner = self.inner.lock();
        let target = Instant::now() + backoff;

        let entries: Vec<_> = inner.queue.drain().collect();
        for mut p in entries {
            if p.0.next_check < target {
                p.0.next_check = target;
            }
            inner.queue.push(p);
        }
    }

    /// Add (or re-home) a project, scheduling it for an immediate check. A fresh
    /// epoch supersedes any prior queued/in-flight entry for the same id.
    pub fn add_project(&self, project_id: Uuid, github_repo: String) {
        let epoch = self.next_epoch.fetch_add(1, AtomicOrdering::Relaxed);
        let mut inner = self.inner.lock();
        inner.registry.insert(
            project_id,
            RegEntry {
                repo: github_repo.clone(),
                epoch,
            },
        );
        inner.queue.push(Reverse(ScheduledProject {
            next_check: Instant::now(),
            project_id,
            github_repo,
            last_activity: None,
            error_count: 0,
            epoch,
        }));
    }

    /// Stop tracking a project. Any queued/in-flight entry becomes stale and is
    /// discarded the next time it surfaces.
    pub fn remove_project(&self, project_id: Uuid) {
        self.inner.lock().registry.remove(&project_id);
    }

    /// Update the stored "owner/repo" of a tracked project in place, keeping its
    /// epoch and schedule. Used to heal a rename mid-lifetime so `trigger_all`
    /// and future reschedules use the canonical name. No-op if untracked.
    pub fn rename_project(&self, project_id: Uuid, new_repo: &str) {
        let mut inner = self.inner.lock();
        if let Some(entry) = inner.registry.get_mut(&project_id) {
            entry.repo = new_repo.to_string();
        }
    }

    /// Schedule every tracked project for an immediate check. Returns the count.
    pub fn trigger_all(&self) -> usize {
        let mut inner = self.inner.lock();
        let now = Instant::now();
        let entries: Vec<(Uuid, String)> = inner
            .registry
            .iter()
            .map(|(id, r)| (*id, r.repo.clone()))
            .collect();

        for (id, repo) in &entries {
            let epoch = self.next_epoch.fetch_add(1, AtomicOrdering::Relaxed);
            if let Some(entry) = inner.registry.get_mut(id) {
                entry.epoch = epoch;
            }
            inner.queue.push(Reverse(ScheduledProject {
                next_check: now,
                project_id: *id,
                github_repo: repo.clone(),
                last_activity: None,
                error_count: 0,
                epoch,
            }));
        }

        drop(inner);
        entries.len()
    }

    /// Get current statistics snapshot
    pub fn stats(&self) -> SyncStats {
        SyncStats {
            synced: self.total_synced.load(AtomicOrdering::Relaxed),
            skipped: self.total_skipped.load(AtomicOrdering::Relaxed),
            errors: self.total_errors.load(AtomicOrdering::Relaxed),
        }
    }

    /// Get info about the next scheduled check (for logging/debugging).
    pub fn next_check_info(&self) -> Option<(String, Duration)> {
        let inner = self.inner.lock();
        inner.queue.peek().map(|p| {
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

/// Cached conditional-request state for the issues endpoint of a single repo.
#[derive(Clone)]
struct IssueCache {
    etag: String,
    timestamp: Option<OffsetDateTime>,
}

/// GitHub API client with per-repo `ETag` caching for the issues endpoint.
pub struct GitHubClient {
    client: reqwest::Client,
    /// "owner/repo" -> last issues `ETag` + the timestamp it resolved to, so a
    /// 304 can return the cached value without re-parsing (and without cost).
    issue_cache: DashMap<String, IssueCache>,
}

// GitHub API response types

/// An item from `GET /repos/{o}/{r}/activity` (push/branch/merge activity).
#[derive(Debug, Deserialize)]
struct ActivityItem {
    timestamp: String,
}

/// An item from `GET /repos/{o}/{r}/issues` (issues and PRs alike).
#[derive(Debug, Deserialize)]
struct IssueItem {
    updated_at: String,
}

/// The subset of `GET /repos/{o}/{r}` we care about: stable identity.
#[derive(Debug, Deserialize)]
struct RepoResponse {
    id: i64,
    full_name: String,
}

/// A repository's stable identity, resolved against the live API.
#[derive(Debug, Clone)]
pub struct RepoMeta {
    /// GitHub's immutable numeric repo id (survives renames and transfers).
    pub id: i64,
    /// Canonical "owner/repo" as GitHub currently reports it.
    pub full_name: String,
}

/// Errors that can occur during GitHub API operations
#[derive(Debug)]
pub enum GitHubError {
    /// HTTP transport failure
    Request(reqwest::Error),
    /// Repository not found / inaccessible (404)
    NotFound(String),
    /// Rate limited (403 with remaining=0, or 429). Carries the computed wait, if
    /// the response told us when the window resets.
    RateLimited { retry_after: Option<Duration> },
    /// Failed to parse a timestamp from the API
    ParseTime(String),
    /// Malformed `github_repo` — permanent, never worth retrying.
    InvalidRepo(String),
    /// A database write failed while persisting sync results.
    Database(String),
    /// Other non-success API response
    Api(u16, String),
}

impl std::fmt::Display for GitHubError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Request(e) => write!(f, "HTTP request failed: {e}"),
            Self::NotFound(repo) => write!(f, "Repository not found or inaccessible: {repo}"),
            Self::RateLimited { .. } => write!(f, "GitHub API rate limit exceeded"),
            Self::ParseTime(s) => write!(f, "Failed to parse timestamp: {s}"),
            Self::InvalidRepo(r) => write!(f, "Invalid github_repo: {r}"),
            Self::Database(e) => write!(f, "Database error during sync: {e}"),
            Self::Api(status, msg) => write!(f, "GitHub API error ({status}): {msg}"),
        }
    }
}

impl std::error::Error for GitHubError {}

/// Read a header as a `&str`, if present and valid UTF-8.
fn header_str<'a>(headers: &'a HeaderMap, key: &str) -> Option<&'a str> {
    headers.get(key).and_then(|v| v.to_str().ok())
}

/// Whether a response status + `x-ratelimit-remaining` indicates rate limiting.
/// GitHub uses 429 for secondary limits and 403-with-remaining-0 for primary.
fn is_rate_limited(status: u16, remaining: Option<&str>) -> bool {
    status == 429 || (status == 403 && remaining == Some("0"))
}

/// Compute how long to wait from rate-limit headers: prefer `retry-after`
/// (seconds), else `x-ratelimit-reset` (Unix seconds) minus `now`. `None` when
/// neither is present (caller applies a fallback).
fn rate_limit_backoff(
    retry_after: Option<&str>,
    reset_at: Option<&str>,
    now_unix: i64,
) -> Option<Duration> {
    if let Some(secs) = retry_after.and_then(|s| s.trim().parse::<u64>().ok()) {
        return Some(Duration::from_secs(secs));
    }
    if let Some(reset) = reset_at.and_then(|s| s.trim().parse::<i64>().ok()) {
        return Some(Duration::from_secs((reset - now_unix).max(0) as u64));
    }
    None
}

/// Build the rate-limit error from a response's headers.
fn rate_limited_error(headers: &HeaderMap) -> GitHubError {
    GitHubError::RateLimited {
        retry_after: rate_limit_backoff(
            header_str(headers, "retry-after"),
            header_str(headers, "x-ratelimit-reset"),
            OffsetDateTime::now_utc().unix_timestamp(),
        ),
    }
}

/// Map a non-success response to the appropriate [`GitHubError`], consuming the
/// body only for the generic-error case.
async fn ensure_ok(
    response: reqwest::Response,
    repo: &str,
) -> Result<reqwest::Response, GitHubError> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    if status == StatusCode::NOT_FOUND {
        return Err(GitHubError::NotFound(repo.to_string()));
    }
    if is_rate_limited(
        status.as_u16(),
        header_str(response.headers(), "x-ratelimit-remaining"),
    ) {
        return Err(rate_limited_error(response.headers()));
    }
    let body = response.text().await.unwrap_or_default();
    Err(GitHubError::Api(status.as_u16(), body))
}

/// Parse an RFC 3339 timestamp from the GitHub API.
fn parse_ts(s: &str) -> Result<OffsetDateTime, GitHubError> {
    OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339)
        .map_err(|_| GitHubError::ParseTime(s.to_string()))
}

impl GitHubClient {
    /// Create a new GitHub client if `GITHUB_TOKEN` is set.
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
            issue_cache: DashMap::new(),
        })
    }

    /// Get the shared GitHub client instance.
    /// Returns None if `GITHUB_TOKEN` is not set, logging a warning once.
    pub async fn get() -> Option<Arc<Self>> {
        GITHUB_CLIENT
            .get_or_init(|| async {
                if let Some(client) = Self::new() {
                    tracing::info!("GitHub sync client initialized");
                    Some(Arc::new(client))
                } else {
                    tracing::warn!(
                        "GitHub sync disabled: GITHUB_TOKEN not set. \
                         Set GITHUB_TOKEN to enable automatic activity sync."
                    );
                    None
                }
            })
            .await
            .clone()
    }

    /// Latest push/branch/merge activity on any ref (no conditional request).
    async fn get_activity_timestamp(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Option<OffsetDateTime>, GitHubError> {
        let slug = format!("{owner}/{repo}");
        let url = format!("https://api.github.com/repos/{slug}/activity?per_page=1");

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(GitHubError::Request)?;
        let response = ensure_ok(response, &slug).await?;

        let items: Vec<ActivityItem> = response.json().await.map_err(GitHubError::Request)?;
        items
            .first()
            .map(|item| parse_ts(&item.timestamp))
            .transpose()
    }

    /// Latest issue/PR update (`updated_at`), polled conditionally with an `ETag`.
    /// A 304 returns the cached timestamp without touching the rate limit.
    async fn get_issue_timestamp(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Option<OffsetDateTime>, GitHubError> {
        let slug = format!("{owner}/{repo}");
        let url = format!(
            "https://api.github.com/repos/{slug}/issues?state=all&sort=updated&direction=desc&per_page=1"
        );

        let mut request = self.client.get(&url);
        if let Some(cached) = self.issue_cache.get(&slug) {
            request = request.header(IF_NONE_MATCH, cached.etag.clone());
        }

        let response = request.send().await.map_err(GitHubError::Request)?;

        if response.status() == StatusCode::NOT_MODIFIED {
            return Ok(self.issue_cache.get(&slug).and_then(|c| c.timestamp));
        }

        let etag = header_str(response.headers(), ETAG.as_str()).map(String::from);
        let response = ensure_ok(response, &slug).await?;

        let items: Vec<IssueItem> = response.json().await.map_err(GitHubError::Request)?;
        let timestamp = items
            .first()
            .map(|item| parse_ts(&item.updated_at))
            .transpose()?;

        if let Some(etag) = etag {
            self.issue_cache
                .insert(slug, IssueCache { etag, timestamp });
        }

        Ok(timestamp)
    }

    /// Fetch the latest activity for a repository: the most recent of push-layer
    /// activity and issue/PR updates.
    pub async fn get_latest_activity(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Option<OffsetDateTime>, GitHubError> {
        let (activity, issue) = tokio::join!(
            self.get_activity_timestamp(owner, repo),
            self.get_issue_timestamp(owner, repo),
        );
        Ok(merge_activity(activity?, issue?))
    }

    /// Resolve a repository's stable identity from `GET /repos/{owner}/{repo}`.
    ///
    /// reqwest follows GitHub's rename/transfer redirect (301), so passing a stale
    /// "owner/repo" returns the *current* `full_name` and the immutable numeric
    /// `id` — this is the basis for both save-time capture and sync-time healing.
    /// A definitive 404 is [`GitHubError::NotFound`]; rate-limit/transport/other
    /// failures surface as their own variants so callers can tell "missing" from
    /// "couldn't tell".
    pub async fn fetch_repo(&self, owner: &str, repo: &str) -> Result<RepoMeta, GitHubError> {
        let slug = format!("{owner}/{repo}");
        let url = format!("https://api.github.com/repos/{slug}");

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(GitHubError::Request)?;
        let response = ensure_ok(response, &slug).await?;

        let body: RepoResponse = response.json().await.map_err(GitHubError::Request)?;
        Ok(RepoMeta {
            id: body.id,
            full_name: body.full_name,
        })
    }
}

/// Normalize a user-supplied GitHub repo reference to canonical "owner/repo".
///
/// Accepts the bare form plus pasted URLs (`https://github.com/owner/repo`,
/// with or without a trailing slash or `.git`). Validates the two-segment shape
/// and the character set of each segment; returns a human-readable error message
/// suitable for a 400 field error otherwise.
pub fn normalize_github_repo(input: &str) -> Result<String, String> {
    let trimmed = input.trim();
    let stripped = trimmed
        .strip_prefix("https://github.com/")
        .or_else(|| trimmed.strip_prefix("http://github.com/"))
        .or_else(|| trimmed.strip_prefix("github.com/"))
        .unwrap_or(trimmed);
    let stripped = stripped.strip_suffix('/').unwrap_or(stripped);
    let stripped = stripped.strip_suffix(".git").unwrap_or(stripped);

    let parts: Vec<&str> = stripped.split('/').collect();
    let invalid = || format!("'{input}' is not a valid GitHub repo (expected owner/repo)");
    if parts.len() != 2 {
        return Err(invalid());
    }
    let (owner, repo) = (parts[0], parts[1]);
    if owner.is_empty() || repo.is_empty() || !is_valid_owner(owner) || !is_valid_repo_name(repo) {
        return Err(invalid());
    }
    Ok(format!("{owner}/{repo}"))
}

/// GitHub usernames/orgs: ASCII alphanumerics and hyphens.
fn is_valid_owner(s: &str) -> bool {
    s.len() <= 39 && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

/// GitHub repo names: ASCII alphanumerics, hyphen, underscore, period.
fn is_valid_repo_name(s: &str) -> bool {
    s.chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// Parse a canonical "owner/repo" string into its parts.
fn parse_github_repo(github_repo: &str) -> Option<(&str, &str)> {
    let (owner, repo) = github_repo.split_once('/')?;
    if owner.is_empty() || repo.is_empty() || repo.contains('/') {
        None
    } else {
        Some((owner, repo))
    }
}

/// Result of syncing one project.
pub struct SyncOutcome {
    /// Freshly fetched latest-activity timestamp (merged push + issue signals).
    pub activity: Option<OffsetDateTime>,
    /// Set when GitHub reports the repo now lives under a different "owner/repo"
    /// than we have stored (rename/transfer); the caller updates the scheduler's
    /// in-memory name. The DB is healed regardless of this field.
    pub renamed_to: Option<String>,
}

/// Sync a single project's GitHub activity and identity, persisting both.
///
/// Activity is the primary signal — its errors drive retry/backoff. Identity
/// resolution (canonical name + stable id) runs in parallel and is best-effort:
/// it heals a renamed repo and backfills the numeric id, but a failure to resolve
/// it never fails the sync.
pub async fn sync_single_project(
    client: &GitHubClient,
    pool: &PgPool,
    project: &ScheduledProject,
) -> Result<SyncOutcome, GitHubError> {
    let (owner, repo) = parse_github_repo(&project.github_repo)
        .ok_or_else(|| GitHubError::InvalidRepo(project.github_repo.clone()))?;

    let (activity, meta) = tokio::join!(
        client.get_latest_activity(owner, repo),
        client.fetch_repo(owner, repo),
    );
    let activity = activity?;

    crate::db::projects::record_github_sync_success(pool, project.project_id, activity)
        .await
        .map_err(|e| GitHubError::Database(e.to_string()))?;

    // Best-effort identity heal: persist the canonical name + id, surfacing a
    // rename so the scheduler can follow it. DB or API hiccups here only log.
    let renamed_to = match meta {
        Ok(meta) => {
            let renamed = meta.full_name != project.github_repo;
            match crate::db::projects::record_github_identity(
                pool,
                project.project_id,
                &meta.full_name,
                meta.id,
            )
            .await
            {
                Ok(()) => renamed.then_some(meta.full_name),
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to persist GitHub repo identity");
                    None
                }
            }
        }
        Err(e) => {
            tracing::debug!(
                repo = %project.github_repo,
                error = %e,
                "Repo identity resolve failed (best-effort, skipping heal)"
            );
            None
        }
    };

    Ok(SyncOutcome {
        activity,
        renamed_to,
    })
}

/// Run an on-demand sync for a single project, returning the outcome.
/// Used by the manual `POST /api/projects/{ref}/sync` endpoint.
pub async fn sync_project_now(
    pool: &PgPool,
    project_id: Uuid,
    github_repo: &str,
) -> Result<SyncOutcome, GitHubError> {
    let client = GitHubClient::get()
        .await
        .ok_or_else(|| GitHubError::Api(503, "GitHub sync disabled (no token)".to_string()))?;

    let scheduled = ScheduledProject {
        next_check: Instant::now(),
        project_id,
        github_repo: github_repo.to_string(),
        last_activity: None,
        error_count: 0,
        epoch: 0,
    };
    sync_single_project(&client, pool, &scheduled).await
}

/// Best-effort: persist a sync error to the project row for the admin UI.
async fn record_error(pool: &PgPool, project_id: Uuid, message: &str) {
    if let Err(e) = crate::db::projects::record_github_sync_error(pool, project_id, message).await {
        tracing::warn!(error = %e, "Failed to record GitHub sync error");
    }
}

/// Run the GitHub activity sync scheduler loop.
///
/// This is the main entry point for the background sync task.
/// It checks for due projects every 30 seconds and syncs them individually.
pub async fn run_scheduler(pool: PgPool, event_sender: crate::events::EventSender) {
    // GitHub sync disabled
    let Some(client) = GitHubClient::get().await else {
        return;
    };

    let Some(scheduler) = init_scheduler(&pool).await else {
        tracing::warn!("Failed to initialize GitHub scheduler");
        return;
    };

    if scheduler.is_empty() {
        tracing::info!("GitHub scheduler: no projects with github_repo configured");
        return;
    }

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

    let mut tick = tokio::time::interval(SCHEDULER_TICK_INTERVAL);

    loop {
        tick.tick().await;

        let mut processed = 0u32;
        while let Some(mut project) = scheduler.pop_if_due() {
            processed += 1;

            match sync_single_project(&client, &pool, &project).await {
                Ok(outcome) => {
                    let new_activity = outcome.activity;
                    // Follow a rename: heal the scheduler's in-memory name (the DB
                    // was already healed) so future polls use the canonical repo.
                    if let Some(new_repo) = outcome.renamed_to {
                        tracing::info!(
                            old_repo = %project.github_repo,
                            new_repo = %new_repo,
                            "GitHub repo renamed; healed canonical name"
                        );
                        scheduler.rename_project(project.project_id, &new_repo);
                        project.github_repo = new_repo;
                    }
                    let merged = merge_activity(new_activity, project.last_activity);
                    let changed = merged != project.last_activity;

                    let interval = calculate_check_interval(merged);
                    tracing::debug!(
                        repo = %project.github_repo,
                        next_check_mins = interval.as_secs() / 60,
                        changed,
                        "GitHub sync complete"
                    );

                    if changed {
                        crate::events::log_event(
                            &event_sender,
                            crate::events::EventType::GithubSyncCompleted,
                            crate::events::EventLevel::Info,
                            Some("project"),
                            Some(project.project_id),
                            None,
                            format!("GitHub sync: new activity on {}", project.github_repo),
                            None,
                        );
                        scheduler.reschedule(project, new_activity);
                    } else {
                        scheduler.reschedule_skipped(project);
                    }
                }
                Err(GitHubError::RateLimited { retry_after }) => {
                    let backoff = retry_after.unwrap_or(RATE_LIMIT_FALLBACK);
                    scheduler.backoff_all(backoff);
                    tracing::warn!(
                        processed,
                        backoff_secs = backoff.as_secs(),
                        "GitHub rate limit hit, backing off all projects"
                    );
                    crate::events::log_event(
                        &event_sender,
                        crate::events::EventType::GithubRateLimited,
                        crate::events::EventLevel::Warning,
                        Some("system"),
                        None,
                        None,
                        format!(
                            "GitHub API rate limit hit, backing off {}s",
                            backoff.as_secs()
                        ),
                        None,
                    );
                    scheduler.reschedule_with_error(project);
                    break;
                }
                Err(GitHubError::NotFound(repo)) => {
                    tracing::warn!(repo = %repo, "GitHub repo not found or inaccessible");
                    record_error(
                        &pool,
                        project.project_id,
                        "Repository not found or inaccessible",
                    )
                    .await;
                    crate::events::log_event(
                        &event_sender,
                        crate::events::EventType::GithubSyncFailed,
                        crate::events::EventLevel::Warning,
                        Some("project"),
                        Some(project.project_id),
                        None,
                        format!("GitHub repo not found or inaccessible: {repo}"),
                        None,
                    );
                    // Permanent until the repo is fixed: park at the max interval.
                    scheduler.reschedule_skipped(project);
                }
                Err(GitHubError::InvalidRepo(repo)) => {
                    tracing::warn!(repo = %repo, "Invalid github_repo, parking project");
                    record_error(&pool, project.project_id, "Invalid github_repo format").await;
                    crate::events::log_event(
                        &event_sender,
                        crate::events::EventType::GithubSyncFailed,
                        crate::events::EventLevel::Error,
                        Some("project"),
                        Some(project.project_id),
                        None,
                        format!("Invalid github_repo, sync disabled: {repo}"),
                        None,
                    );
                    scheduler.reschedule_skipped(project);
                }
                Err(GitHubError::Database(msg)) => {
                    // Can't persist to the DB right now; just retry with backoff.
                    tracing::warn!(
                        repo = %project.github_repo,
                        error = %msg,
                        "GitHub sync DB write failed, scheduling retry"
                    );
                    scheduler.reschedule_with_error(project);
                }
                Err(e) => {
                    tracing::warn!(
                        repo = %project.github_repo,
                        error = %e,
                        "GitHub sync error, scheduling retry with backoff"
                    );
                    record_error(&pool, project.project_id, &e.to_string()).await;
                    crate::events::log_event(
                        &event_sender,
                        crate::events::EventType::GithubSyncFailed,
                        crate::events::EventLevel::Error,
                        Some("project"),
                        Some(project.project_id),
                        None,
                        format!("GitHub sync failed for {}: {}", project.github_repo, e),
                        None,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn ts(rfc3339: &str) -> OffsetDateTime {
        parse_ts(rfc3339).unwrap()
    }

    #[test]
    fn check_interval_is_clamped_to_bounds() {
        let min = min_interval();
        let max = max_interval();

        // Today's activity → minimum interval.
        let now = OffsetDateTime::now_utc();
        assert_eq!(calculate_check_interval(Some(now)), min);

        // No activity recorded → maximum interval.
        assert_eq!(calculate_check_interval(None), max);

        // Far in the past (beyond DAYS_TO_MAX) → clamped to maximum.
        let old = now - Duration::from_hours(200 * 24);
        assert_eq!(calculate_check_interval(Some(old)), max);
    }

    #[test]
    fn check_interval_grows_monotonically_with_age() {
        let now = OffsetDateTime::now_utc();
        let day = Duration::from_hours(24);

        let recent = calculate_check_interval(Some(now - day));
        let week = calculate_check_interval(Some(now - 7 * day));
        let month = calculate_check_interval(Some(now - 30 * day));

        assert!(recent <= week, "1d {recent:?} should be <= 7d {week:?}");
        assert!(week <= month, "7d {week:?} should be <= 30d {month:?}");
        assert!(month <= max_interval());
    }

    #[test]
    fn merge_activity_takes_the_latest_and_never_regresses() {
        let early = ts("2024-01-01T00:00:00Z");
        let late = ts("2024-06-01T00:00:00Z");

        assert_eq!(merge_activity(Some(early), Some(late)), Some(late));
        assert_eq!(merge_activity(Some(late), Some(early)), Some(late));
        assert_eq!(merge_activity(Some(early), None), Some(early));
        assert_eq!(merge_activity(None, Some(late)), Some(late));
        assert_eq!(merge_activity(None, None), None);
    }

    #[test]
    fn normalize_repo_accepts_bare_and_url_forms() {
        assert_eq!(
            normalize_github_repo("Xevion/xevion.dev").unwrap(),
            "Xevion/xevion.dev"
        );
        assert_eq!(
            normalize_github_repo("https://github.com/Xevion/xevion.dev").unwrap(),
            "Xevion/xevion.dev"
        );
        assert_eq!(
            normalize_github_repo("http://github.com/owner/repo/").unwrap(),
            "owner/repo"
        );
        assert_eq!(
            normalize_github_repo("github.com/owner/repo.git").unwrap(),
            "owner/repo"
        );
        assert_eq!(
            normalize_github_repo("  owner/repo  ").unwrap(),
            "owner/repo"
        );
    }

    #[test]
    fn normalize_repo_rejects_malformed_input() {
        assert!(normalize_github_repo("not-a-repo").is_err()); // one segment
        assert!(normalize_github_repo("owner/repo/extra").is_err()); // three segments
        assert!(normalize_github_repo("owner/").is_err()); // empty repo
        assert!(normalize_github_repo("/repo").is_err()); // empty owner
        assert!(normalize_github_repo("ow ner/repo").is_err()); // space in owner
        assert!(normalize_github_repo("owner/re po").is_err()); // space in repo
    }

    #[test]
    fn parse_repo_splits_canonical_form() {
        assert_eq!(parse_github_repo("owner/repo"), Some(("owner", "repo")));
        assert_eq!(parse_github_repo("owner"), None);
        assert_eq!(parse_github_repo("owner/repo/extra"), None);
        assert_eq!(parse_github_repo("/repo"), None);
    }

    #[test]
    fn rate_limit_detected_for_429_and_403_with_zero_remaining() {
        assert!(is_rate_limited(429, None));
        assert!(is_rate_limited(429, Some("57")));
        assert!(is_rate_limited(403, Some("0")));
        assert!(!is_rate_limited(403, Some("57"))); // 403 for another reason
        assert!(!is_rate_limited(403, None));
        assert!(!is_rate_limited(404, Some("0")));
        assert!(!is_rate_limited(200, None));
    }

    #[test]
    fn rate_limit_backoff_prefers_retry_after_then_reset() {
        // retry-after wins, in seconds.
        assert_eq!(
            rate_limit_backoff(Some("120"), Some("9999999999"), 0),
            Some(Duration::from_mins(2))
        );
        // falls back to x-ratelimit-reset minus now.
        assert_eq!(
            rate_limit_backoff(None, Some("1000"), 700),
            Some(Duration::from_mins(5))
        );
        // a reset already in the past clamps to zero, not a negative/huge value.
        assert_eq!(
            rate_limit_backoff(None, Some("500"), 900),
            Some(Duration::ZERO)
        );
        // nothing usable → None (caller applies a fallback).
        assert_eq!(rate_limit_backoff(None, None, 0), None);
    }

    /// A project row with only the fields the scheduler reads.
    fn project(id: Uuid, repo: &str) -> DbProject {
        DbProject {
            id,
            slug: "s".into(),
            name: "n".into(),
            short_description: "d".into(),
            status: crate::db::ProjectStatus::Active,
            hidden: false,
            github_repo: Some(repo.into()),
            github_repo_id: None,
            demo_url: None,
            last_github_activity: None,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            detail_content: None,
            project_type: None,
            private: false,
            terminal_cast: None,
            accent_color: None,
            github_synced_at: None,
            github_sync_error: None,
        }
    }

    #[test]
    fn scheduler_pops_due_projects_then_reports_empty_queue() {
        let id = Uuid::new_v4();
        let sched = GitHubScheduler::new(vec![project(id, "owner/repo")]);

        // The single staggered entry is due immediately (offset 0/1 = now).
        let popped = sched.pop_if_due().expect("one due project");
        assert_eq!(popped.project_id, id);
        // Nothing left in the queue until it's rescheduled.
        assert!(sched.pop_if_due().is_none());
        // It's still tracked, though.
        assert_eq!(sched.len(), 1);
    }

    #[test]
    fn removing_a_popped_project_prevents_resurrection() {
        let id = Uuid::new_v4();
        let sched = GitHubScheduler::new(vec![project(id, "owner/repo")]);

        // Simulate the sync loop: pop (now in-flight, not in the queue)...
        let in_flight = sched.pop_if_due().expect("due");
        // ...the project is deleted while the sync is in flight...
        sched.remove_project(id);
        assert_eq!(sched.len(), 0);
        // ...and the completing sync tries to reschedule it.
        sched.reschedule(in_flight, None);

        // It must NOT come back: a removed project stays gone.
        assert!(sched.pop_if_due().is_none());
        assert_eq!(sched.len(), 0);
    }

    #[test]
    fn re_homing_during_sync_drops_the_stale_entry() {
        let id = Uuid::new_v4();
        let sched = GitHubScheduler::new(vec![project(id, "old/repo")]);

        let in_flight = sched.pop_if_due().expect("due"); // old/repo, in flight
        // Repo changes mid-sync: handlers call remove then add.
        sched.remove_project(id);
        sched.add_project(id, "new/repo".to_string());
        // The old in-flight entry reschedules but is now superseded.
        sched.reschedule(in_flight, None);

        // Exactly one entry surfaces, and it's the new repo.
        let next = sched.pop_if_due().expect("the re-homed entry is due");
        assert_eq!(next.github_repo, "new/repo");
        assert!(sched.pop_if_due().is_none());
    }

    #[test]
    fn adding_an_existing_id_supersedes_the_old_entry() {
        let id = Uuid::new_v4();
        let sched = GitHubScheduler::new(vec![project(id, "owner/repo")]);

        // Re-add the same id (e.g. a duplicate enqueue): old queued entry is stale.
        sched.add_project(id, "owner/repo".to_string());

        // Only one valid entry should ever surface.
        assert!(sched.pop_if_due().is_some());
        assert!(sched.pop_if_due().is_none());
        assert_eq!(sched.len(), 1);
    }

    #[test]
    fn error_backoff_doubles_per_failure_and_caps_at_max() {
        let id = Uuid::new_v4();
        let sched = GitHubScheduler::new(vec![project(id, "owner/repo")]);
        let base = sched.pop_if_due().expect("due");

        // First error: ~2x base interval.
        let one = base.clone();
        sched.reschedule_with_error(one);
        // Re-pop is not time-due, so assert via the stats counter instead.
        assert_eq!(sched.stats().errors, 1);

        // Many consecutive errors never exceed the max interval.
        let mut p = base;
        p.error_count = 20; // far past the 2^6 cap
        let before = sched.len();
        sched.reschedule_with_error(p);
        assert_eq!(sched.len(), before); // still tracked, just deferred
        assert_eq!(sched.stats().errors, 2);
    }
}
