use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::fmt;
use std::sync::Arc;
use time::{Duration, OffsetDateTime};
use ts_rs::TS;
use ulid::Ulid;

/// Browser sessions are short-lived; the cookie is refreshed on each login.
const SESSION_DURATION_DAYS: i64 = 7;
/// CLI tokens are long-lived and slide forward on use (see `validate_bearer`).
const CLI_SESSION_DURATION_DAYS: i64 = 90;
/// Don't persist a sliding-expiry bump more often than this, to keep authn cheap.
const CLI_SLIDE_THROTTLE: Duration = Duration::hours(6);
/// Browser sessions don't slide their expiry, but we still record activity so
/// the admin UI shows a real "last active". Throttled to avoid a write per page.
const BROWSER_TOUCH_THROTTLE: Duration = Duration::minutes(5);

/// How a session authenticates: an interactive browser cookie, or a long-lived
/// CLI bearer token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum SessionType {
    Browser,
    Cli,
}

impl SessionType {
    /// Canonical wire/storage string. Single source of truth for the column
    /// values and the `Display` impl.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Browser => "browser",
            Self::Cli => "cli",
        }
    }

    fn from_db(s: &str) -> Self {
        match s {
            "cli" => Self::Cli,
            _ => Self::Browser,
        }
    }
}

impl fmt::Display for SessionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
pub struct Session {
    pub id: Ulid,
    pub user_id: i32,
    pub username: String,
    pub created_at: OffsetDateTime,
    pub last_active_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
    pub session_type: SessionType,
    /// Human-readable label for CLI tokens (e.g. device hostname). `None` for browsers.
    pub label: Option<String>,
}

/// Row shape for loading sessions from the database.
#[derive(sqlx::FromRow)]
struct SessionRow {
    id: String,
    user_id: i32,
    username: String,
    created_at: OffsetDateTime,
    last_active_at: OffsetDateTime,
    expires_at: OffsetDateTime,
    session_type: String,
    label: Option<String>,
    token_hash: Option<String>,
}

/// SHA-256 hex digest of a raw CLI token. Only the hash is ever persisted.
pub fn hash_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

/// Mint a fresh opaque CLI token. The `xev_` prefix makes it greppable in logs
/// and recognizable if it leaks.
fn generate_cli_token() -> String {
    format!("xev_{}", nanoid::nanoid!(40))
}

#[derive(Debug, Clone)]
pub struct AdminUser {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
}

#[derive(Clone)]
pub struct SessionManager {
    sessions: Arc<DashMap<Ulid, Session>>,
    /// Maps a CLI token's SHA-256 hash to its session id for bearer lookup.
    token_index: Arc<DashMap<String, Ulid>>,
    pool: PgPool,
}

impl SessionManager {
    pub async fn new(pool: PgPool) -> Result<Self, sqlx::Error> {
        let manager = Self {
            sessions: Arc::new(DashMap::new()),
            token_index: Arc::new(DashMap::new()),
            pool,
        };

        manager.load_active_sessions().await?;

        Ok(manager)
    }

    async fn load_active_sessions(&self) -> Result<(), sqlx::Error> {
        let now = OffsetDateTime::now_utc();

        let sessions: Vec<SessionRow> = sqlx::query_as(
            r"
            SELECT s.id, s.user_id, u.username, s.created_at, s.last_active_at,
                   s.expires_at, s.session_type, s.label, s.token_hash
            FROM sessions s
            JOIN admin_users u ON s.user_id = u.id
            WHERE s.expires_at > $1
            ",
        )
        .bind(now)
        .fetch_all(&self.pool)
        .await?;

        for SessionRow {
            id: id_str,
            user_id,
            username,
            created_at,
            last_active_at,
            expires_at,
            session_type: ty,
            label,
            token_hash,
        } in sessions
        {
            if let Ok(id) = Ulid::from_string(&id_str) {
                let session = Session {
                    id,
                    user_id,
                    username,
                    created_at,
                    last_active_at,
                    expires_at,
                    session_type: SessionType::from_db(&ty),
                    label,
                };
                if let Some(hash) = token_hash {
                    self.token_index.insert(hash, id);
                }
                self.sessions.insert(id, session);
            }
        }

        tracing::info!(
            session_count = self.sessions.len(),
            "Loaded active sessions from database"
        );

        Ok(())
    }

    pub async fn create_session(
        &self,
        user_id: i32,
        username: String,
    ) -> Result<Session, sqlx::Error> {
        let id = Ulid::new();
        let created_at = OffsetDateTime::now_utc();
        let expires_at = created_at + Duration::days(SESSION_DURATION_DAYS);

        sqlx::query(
            r"
            INSERT INTO sessions (id, user_id, created_at, expires_at, last_active_at, session_type)
            VALUES ($1, $2, $3, $4, $5, $6)
            ",
        )
        .bind(id.to_string())
        .bind(user_id)
        .bind(created_at)
        .bind(expires_at)
        .bind(created_at)
        .bind(SessionType::Browser.as_str())
        .execute(&self.pool)
        .await?;

        let session = Session {
            id,
            user_id,
            username,
            created_at,
            last_active_at: created_at,
            expires_at,
            session_type: SessionType::Browser,
            label: None,
        };

        self.sessions.insert(id, session.clone());

        tracing::debug!(session_id = %id, user_id, "Created session");

        Ok(session)
    }

    /// Create a long-lived CLI session, returning the session and the raw token.
    /// The raw token is shown to the client exactly once; only its hash persists.
    pub async fn create_cli_session(
        &self,
        user_id: i32,
        username: String,
        label: Option<String>,
    ) -> Result<(Session, String), sqlx::Error> {
        let id = Ulid::new();
        let token = generate_cli_token();
        let token_hash = hash_token(&token);
        let created_at = OffsetDateTime::now_utc();
        let expires_at = created_at + Duration::days(CLI_SESSION_DURATION_DAYS);

        sqlx::query(
            r"
            INSERT INTO sessions
                (id, user_id, created_at, expires_at, last_active_at, session_type, token_hash, label)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ",
        )
        .bind(id.to_string())
        .bind(user_id)
        .bind(created_at)
        .bind(expires_at)
        .bind(created_at)
        .bind(SessionType::Cli.as_str())
        .bind(&token_hash)
        .bind(&label)
        .execute(&self.pool)
        .await?;

        let session = Session {
            id,
            user_id,
            username,
            created_at,
            last_active_at: created_at,
            expires_at,
            session_type: SessionType::Cli,
            label,
        };

        self.token_index.insert(token_hash, id);
        self.sessions.insert(id, session.clone());

        tracing::info!(session_id = %id, user_id, "Created CLI session");

        Ok((session, token))
    }

    pub fn get_session(&self, session_id: Ulid) -> Option<Session> {
        self.sessions.get(&session_id).map(|s| s.clone())
    }

    pub fn validate_session(&self, session_id: Ulid) -> Option<Session> {
        let session = self.get_session(session_id)?;

        if session.expires_at < OffsetDateTime::now_utc() {
            self.sessions.remove(&session_id);
            return None;
        }

        Some(session)
    }

    /// Validate a CLI bearer token. On success, slides the expiry forward (at
    /// most once per `CLI_SLIDE_THROTTLE` to avoid a write per request).
    pub async fn validate_bearer(&self, token: &str) -> Option<Session> {
        let token_hash = hash_token(token);
        let session_id = *self.token_index.get(&token_hash)?;
        let session = self.validate_session(session_id)?;

        let now = OffsetDateTime::now_utc();
        if now - session.last_active_at >= CLI_SLIDE_THROTTLE {
            let expires_at = now + Duration::days(CLI_SESSION_DURATION_DAYS);
            if let Err(e) = sqlx::query(
                "UPDATE sessions SET last_active_at = $1, expires_at = $2 WHERE id = $3",
            )
            .bind(now)
            .bind(expires_at)
            .bind(session_id.to_string())
            .execute(&self.pool)
            .await
            {
                tracing::warn!(error = %e, session_id = %session_id, "Failed to slide CLI session expiry");
            } else if let Some(mut entry) = self.sessions.get_mut(&session_id) {
                entry.last_active_at = now;
                entry.expires_at = expires_at;
            }
        }

        Some(session)
    }

    /// Record activity on a browser session. Unlike CLI tokens this does not
    /// slide the expiry (the 7-day window is fixed at login); it only refreshes
    /// `last_active_at`, throttled so navigation doesn't write on every request.
    pub async fn touch_session(&self, session_id: Ulid) {
        let now = OffsetDateTime::now_utc();

        match self.sessions.get(&session_id) {
            Some(session) if now - session.last_active_at < BROWSER_TOUCH_THROTTLE => return,
            Some(_) => {}
            None => return,
        }

        if let Err(e) = sqlx::query("UPDATE sessions SET last_active_at = $1 WHERE id = $2")
            .bind(now)
            .bind(session_id.to_string())
            .execute(&self.pool)
            .await
        {
            tracing::warn!(error = %e, session_id = %session_id, "Failed to record session activity");
        } else if let Some(mut entry) = self.sessions.get_mut(&session_id) {
            entry.last_active_at = now;
        }
    }

    /// All active sessions for a user, newest first. Used by the admin UI.
    pub fn list_user_sessions(&self, user_id: i32) -> Vec<Session> {
        let mut sessions: Vec<Session> = self
            .sessions
            .iter()
            .filter(|s| s.user_id == user_id)
            .map(|s| s.clone())
            .collect();
        sessions.sort_by_key(|s| std::cmp::Reverse(s.created_at));
        sessions
    }

    pub async fn delete_session(&self, session_id: Ulid) -> Result<(), sqlx::Error> {
        if let Some((_, session)) = self.sessions.remove(&session_id)
            && session.session_type == SessionType::Cli
        {
            self.token_index.retain(|_, id| *id != session_id);
        }

        sqlx::query("DELETE FROM sessions WHERE id = $1")
            .bind(session_id.to_string())
            .execute(&self.pool)
            .await?;

        tracing::debug!(session_id = %session_id, "Deleted session");

        Ok(())
    }

    pub async fn cleanup_expired(&self) -> Result<usize, sqlx::Error> {
        let now = OffsetDateTime::now_utc();

        let result = sqlx::query("DELETE FROM sessions WHERE expires_at < $1")
            .bind(now)
            .execute(&self.pool)
            .await?;

        let expired_count = result.rows_affected() as usize;

        self.sessions.retain(|_, session| session.expires_at >= now);
        self.token_index
            .retain(|_, id| self.sessions.contains_key(id));

        if expired_count > 0 {
            tracing::info!(expired_count, "Cleaned up expired sessions");
        }

        Ok(expired_count)
    }
}

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(password_hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    let argon2 = Argon2::default();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(e),
    }
}

pub async fn get_admin_user(
    pool: &PgPool,
    username: &str,
) -> Result<Option<AdminUser>, sqlx::Error> {
    let user: Option<(i32, String, String)> = sqlx::query_as(
        r"
        SELECT id, username, password_hash
        FROM admin_users
        WHERE username = $1
        ",
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;

    Ok(user.map(|(id, username, password_hash)| AdminUser {
        id,
        username,
        password_hash,
    }))
}

pub async fn create_admin_user(
    pool: &PgPool,
    username: &str,
    password: &str,
) -> Result<i32, Box<dyn std::error::Error>> {
    let password_hash =
        hash_password(password).map_err(|e| format!("Failed to hash password: {e}"))?;

    let (id,): (i32,) = sqlx::query_as(
        r"
        INSERT INTO admin_users (username, password_hash)
        VALUES ($1, $2)
        RETURNING id
        ",
    )
    .bind(username)
    .bind(password_hash)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

pub async fn ensure_admin_user(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let username = std::env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string());
    let password = std::env::var("ADMIN_PASSWORD")
        .map_err(|_| "ADMIN_PASSWORD environment variable must be set")?;

    if get_admin_user(pool, &username).await?.is_none() {
        create_admin_user(pool, &username, &password).await?;
        tracing::info!(username, "Created admin user");
    } else {
        tracing::debug!(username, "Admin user already exists");
    }

    Ok(())
}

/// Check if the request has a valid admin session cookie (from `AppState`).
pub fn check_session(
    state: &crate::state::AppState,
    jar: &axum_extra::extract::CookieJar,
) -> Option<Session> {
    let session_cookie = jar.get("admin_session")?;
    let session_id = ulid::Ulid::from_string(session_cookie.value()).ok()?;
    state.session_manager.validate_session(session_id)
}

/// Extract a CLI bearer token from an `Authorization: Bearer <token>` header.
pub fn bearer_token(headers: &axum::http::HeaderMap) -> Option<String> {
    let value = headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?;
    let token = value.strip_prefix("Bearer ")?.trim();
    (!token.is_empty()).then(|| token.to_string())
}

/// Authenticate a request via either the browser session cookie or a CLI
/// bearer token. Cookie is checked first (cheaper, no sliding-expiry write).
pub async fn authenticate(
    state: &crate::state::AppState,
    headers: &axum::http::HeaderMap,
) -> Option<Session> {
    let jar = axum_extra::extract::CookieJar::from_headers(headers);
    if let Some(session) = check_session(state, &jar) {
        return Some(session);
    }
    let token = bearer_token(headers)?;
    state.session_manager.validate_bearer(&token).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_token_is_deterministic_hex() {
        let a = hash_token("xev_example");
        let b = hash_token("xev_example");
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a, hash_token("xev_other"));
    }

    #[test]
    fn generated_tokens_are_prefixed_and_unique() {
        let a = generate_cli_token();
        let b = generate_cli_token();
        assert!(a.starts_with("xev_"));
        assert_ne!(a, b);
    }

    #[test]
    fn bearer_token_parses_authorization_header() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            "Bearer xev_abc123".parse().unwrap(),
        );
        assert_eq!(bearer_token(&headers).as_deref(), Some("xev_abc123"));

        let empty = axum::http::HeaderMap::new();
        assert_eq!(bearer_token(&empty), None);
    }
}
