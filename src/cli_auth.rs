//! Device-authorization flow for the CLI.
//!
//! The CLI starts a request and is handed a short, human-readable user code. It
//! opens the approval page in a browser and waits on an SSE stream. An
//! authenticated admin confirms the code matches, which mints a long-lived CLI
//! token and pushes it back over the stream. Pending requests live in memory
//! only — they're short-lived and losing them on restart just means re-running
//! `xevion api login`.

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use time::{Duration, OffsetDateTime};
use tokio::sync::watch;
use ulid::Ulid;

/// Path of the browser approval page; the CLI prepends the target API base URL.
pub const VERIFICATION_PATH: &str = "/admin/authorize";

/// `POST /api/auth/device/start` request body.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartRequest {
    /// Human-readable device label (typically the hostname).
    pub label: Option<String>,
}

/// `POST /api/auth/device/start` response body.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartResponse {
    pub request_id: String,
    pub user_code: String,
    /// Relative path of the approval page; combine with the API base URL.
    pub verification_path: String,
    #[serde(with = "time::serde::rfc3339")]
    pub expires_at: OffsetDateTime,
}

/// `POST /api/auth/device/approve` request body.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApproveRequest {
    pub request_id: String,
    pub user_code: String,
}

/// `POST /api/auth/device/deny` request body.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DenyRequest {
    pub request_id: String,
}

/// How long an unconfirmed request stays valid.
const REQUEST_TTL: Duration = Duration::minutes(10);

/// Unambiguous alphabet for the user code (no 0/O/1/I/L).
const CODE_ALPHABET: [char; 31] = [
    '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'M',
    'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

/// Status of a pending request: held in the `watch` channel, serialized verbatim
/// as the SSE payload to the waiting CLI, and deserialized by the CLI client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "status")]
pub enum CliAuthStatus {
    Pending,
    Approved { token: String, username: String },
    Denied,
}

struct PendingAuth {
    user_code: String,
    label: Option<String>,
    expires_at: OffsetDateTime,
    tx: watch::Sender<CliAuthStatus>,
}

/// Public view of a pending request, for the approval page.
#[derive(Debug, Clone)]
pub struct PendingInfo {
    pub user_code: String,
    pub label: Option<String>,
    pub expires_at: OffsetDateTime,
}

/// Outcome of starting a new request.
#[derive(Debug, Clone)]
pub struct StartedRequest {
    pub request_id: Ulid,
    pub user_code: String,
    pub expires_at: OffsetDateTime,
}

#[derive(Clone, Default)]
pub struct CliAuthRegistry {
    requests: Arc<DashMap<Ulid, PendingAuth>>,
}

impl CliAuthRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    fn generate_user_code() -> String {
        let chars: String = (0..8)
            .map(|_| CODE_ALPHABET[rand::random_range(0..CODE_ALPHABET.len())])
            .collect();
        format!("{}-{}", &chars[..4], &chars[4..])
    }

    /// Register a new pending request.
    pub fn start(&self, label: Option<String>) -> StartedRequest {
        self.sweep_expired();

        let request_id = Ulid::new();
        let user_code = Self::generate_user_code();
        let expires_at = OffsetDateTime::now_utc() + REQUEST_TTL;
        let (tx, _rx) = watch::channel(CliAuthStatus::Pending);

        self.requests.insert(
            request_id,
            PendingAuth {
                user_code: user_code.clone(),
                label,
                expires_at,
                tx,
            },
        );

        StartedRequest {
            request_id,
            user_code,
            expires_at,
        }
    }

    /// Metadata for the approval page. `None` if unknown or expired.
    pub fn info(&self, request_id: Ulid) -> Option<PendingInfo> {
        let entry = self.requests.get(&request_id)?;
        if entry.expires_at < OffsetDateTime::now_utc() {
            return None;
        }
        Some(PendingInfo {
            user_code: entry.user_code.clone(),
            label: entry.label.clone(),
            expires_at: entry.expires_at,
        })
    }

    /// Subscribe to status changes for the SSE waiter. `None` if unknown/expired.
    pub fn subscribe(&self, request_id: Ulid) -> Option<watch::Receiver<CliAuthStatus>> {
        let entry = self.requests.get(&request_id)?;
        if entry.expires_at < OffsetDateTime::now_utc() {
            return None;
        }
        Some(entry.tx.subscribe())
    }

    /// Approve a request, pushing the minted token to the waiter. The supplied
    /// `user_code` must match (anti-phishing). Returns `false` if the request is
    /// unknown, expired, or the code mismatches.
    pub fn approve(
        &self,
        request_id: Ulid,
        user_code: &str,
        token: String,
        username: String,
    ) -> bool {
        let Some(entry) = self.requests.get(&request_id) else {
            return false;
        };
        if entry.expires_at < OffsetDateTime::now_utc()
            || !entry.user_code.eq_ignore_ascii_case(user_code.trim())
        {
            return false;
        }
        let _ = entry.tx.send(CliAuthStatus::Approved { token, username });
        true
    }

    /// Deny a request. Returns `false` if unknown or expired.
    pub fn deny(&self, request_id: Ulid) -> bool {
        let Some(entry) = self.requests.get(&request_id) else {
            return false;
        };
        let _ = entry.tx.send(CliAuthStatus::Denied);
        true
    }

    fn sweep_expired(&self) {
        let now = OffsetDateTime::now_utc();
        self.requests.retain(|_, entry| entry.expires_at >= now);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_code_has_grouped_format() {
        let code = CliAuthRegistry::generate_user_code();
        assert_eq!(code.len(), 9);
        assert_eq!(&code[4..5], "-");
        assert!(
            code.chars()
                .filter(|c| *c != '-')
                .all(|c| CODE_ALPHABET.contains(&c))
        );
    }

    #[test]
    fn approve_requires_matching_code() {
        let registry = CliAuthRegistry::new();
        let started = registry.start(Some("laptop".into()));

        // Wrong code is rejected.
        assert!(!registry.approve(
            started.request_id,
            "WRONG-CODE",
            "tok".into(),
            "admin".into()
        ));

        // Correct code (case-insensitive) approves and pushes the token.
        let mut rx = registry.subscribe(started.request_id).unwrap();
        assert!(registry.approve(
            started.request_id,
            &started.user_code.to_lowercase(),
            "tok".into(),
            "admin".into(),
        ));
        assert_eq!(
            *rx.borrow_and_update(),
            CliAuthStatus::Approved {
                token: "tok".into(),
                username: "admin".into()
            }
        );
    }

    #[test]
    fn deny_sets_denied_status() {
        let registry = CliAuthRegistry::new();
        let started = registry.start(None);
        let rx = registry.subscribe(started.request_id).unwrap();
        assert!(registry.deny(started.request_id));
        assert_eq!(*rx.borrow(), CliAuthStatus::Denied);
    }

    #[test]
    fn unknown_request_yields_nothing() {
        let registry = CliAuthRegistry::new();
        let missing = Ulid::new();
        assert!(registry.info(missing).is_none());
        assert!(registry.subscribe(missing).is_none());
        assert!(!registry.deny(missing));
    }
}
