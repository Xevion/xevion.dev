use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use time::{Duration, OffsetDateTime};
use ulid::Ulid;

const SESSION_DURATION_DAYS: i64 = 7;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Ulid,
    pub user_id: i32,
    pub username: String,
    pub expires_at: OffsetDateTime,
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
    pool: PgPool,
}

impl SessionManager {
    pub async fn new(pool: PgPool) -> Result<Self, sqlx::Error> {
        let manager = Self {
            sessions: Arc::new(DashMap::new()),
            pool,
        };

        manager.load_active_sessions().await?;

        Ok(manager)
    }

    async fn load_active_sessions(&self) -> Result<(), sqlx::Error> {
        let now = OffsetDateTime::now_utc();

        let sessions: Vec<(String, i32, String, OffsetDateTime)> = sqlx::query_as(
            r#"
            SELECT s.id, s.user_id, u.username, s.expires_at
            FROM sessions s
            JOIN admin_users u ON s.user_id = u.id
            WHERE s.expires_at > $1
            "#,
        )
        .bind(now)
        .fetch_all(&self.pool)
        .await?;

        for (id_str, user_id, username, expires_at) in sessions {
            if let Ok(id) = Ulid::from_string(&id_str) {
                let session = Session {
                    id,
                    user_id,
                    username,
                    expires_at,
                };
                self.sessions.insert(id, session);
            }
        }

        tracing::info!(
            "Loaded {} active sessions from database",
            self.sessions.len()
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
            r#"
            INSERT INTO sessions (id, user_id, created_at, expires_at, last_active_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(id.to_string())
        .bind(user_id)
        .bind(created_at)
        .bind(expires_at)
        .bind(created_at)
        .execute(&self.pool)
        .await?;

        let session = Session {
            id,
            user_id,
            username,
            expires_at,
        };

        self.sessions.insert(id, session.clone());

        tracing::debug!("Created session {} for user {}", id, user_id);

        Ok(session)
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

    pub async fn delete_session(&self, session_id: Ulid) -> Result<(), sqlx::Error> {
        self.sessions.remove(&session_id);

        sqlx::query("DELETE FROM sessions WHERE id = $1")
            .bind(session_id.to_string())
            .execute(&self.pool)
            .await?;

        tracing::debug!("Deleted session {}", session_id);

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

        if expired_count > 0 {
            tracing::info!("Cleaned up {} expired sessions", expired_count);
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
        r#"
        SELECT id, username, password_hash
        FROM admin_users
        WHERE username = $1
        "#,
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
        hash_password(password).map_err(|e| format!("Failed to hash password: {}", e))?;

    let (id,): (i32,) = sqlx::query_as(
        r#"
        INSERT INTO admin_users (username, password_hash)
        VALUES ($1, $2)
        RETURNING id
        "#,
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
        tracing::info!("Created admin user: {}", username);
    } else {
        tracing::debug!("Admin user '{}' already exists", username);
    }

    Ok(())
}

/// Check if the request has a valid admin session (from AppState)
pub fn check_session(
    state: &crate::state::AppState,
    jar: &axum_extra::extract::CookieJar,
) -> Option<Session> {
    let session_cookie = jar.get("admin_session")?;
    let session_id = ulid::Ulid::from_string(session_cookie.value()).ok()?;
    state.session_manager.validate_session(session_id)
}

/// Return a 401 Unauthorized response for API endpoints
pub fn require_auth_response() -> impl axum::response::IntoResponse {
    use axum::{Json, http::StatusCode};

    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({
            "error": "Unauthorized",
            "message": "Authentication required"
        })),
    )
}
