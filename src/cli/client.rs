use reqwest::{Client, Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::path::Path;
use time::OffsetDateTime;

/// Session data stored in the session file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub api_url: String,
    pub session_token: String,
    pub username: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub expires_at: OffsetDateTime,
}

/// API client with session management
pub struct ApiClient {
    pub api_url: String,
    session_path: String,
    session: Option<Session>,
    client: Client,
}

#[derive(Debug)]
pub enum ApiError {
    /// HTTP request failed
    Request(reqwest::Error),
    /// Server returned an error response
    Http { status: StatusCode, body: String },
    /// Failed to parse response
    Parse(String),
    /// Session file error
    Session(String),
    /// Not authenticated
    Unauthorized,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Request(e) => write!(f, "Request failed: {}", e),
            ApiError::Http { status, body } => {
                write!(f, "HTTP {}: {}", status, body)
            }
            ApiError::Parse(msg) => write!(f, "Parse error: {}", msg),
            ApiError::Session(msg) => write!(f, "Session error: {}", msg),
            ApiError::Unauthorized => write!(f, "Not authenticated. Run 'xevion api login' first."),
        }
    }
}

impl std::error::Error for ApiError {}

impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self {
        ApiError::Request(e)
    }
}

impl ApiClient {
    /// Create a new API client
    pub fn new(api_url: String, session_path: String) -> Self {
        let session = Self::load_session_from_path(&session_path);
        Self {
            api_url,
            session_path,
            session,
            client: Client::new(),
        }
    }

    fn load_session_from_path(path: &str) -> Option<Session> {
        let path = Path::new(path);
        if !path.exists() {
            return None;
        }

        let content = std::fs::read_to_string(path).ok()?;
        let session: Session = serde_json::from_str(&content).ok()?;

        // Check if session is expired
        if session.expires_at < OffsetDateTime::now_utc() {
            return None;
        }

        Some(session)
    }

    /// Save session to file
    pub fn save_session(&mut self, session: Session) -> Result<(), ApiError> {
        let content = serde_json::to_string_pretty(&session)
            .map_err(|e| ApiError::Session(format!("Failed to serialize session: {}", e)))?;

        std::fs::write(&self.session_path, content)
            .map_err(|e| ApiError::Session(format!("Failed to write session file: {}", e)))?;

        self.session = Some(session);
        Ok(())
    }

    /// Clear the session
    pub fn clear_session(&mut self) -> Result<(), ApiError> {
        let path = Path::new(&self.session_path);
        if path.exists() {
            std::fs::remove_file(path)
                .map_err(|e| ApiError::Session(format!("Failed to remove session file: {}", e)))?;
        }
        self.session = None;
        Ok(())
    }

    /// Get current session if valid
    pub fn session(&self) -> Option<&Session> {
        self.session.as_ref()
    }

    /// Check if we have a valid session
    pub fn is_authenticated(&self) -> bool {
        self.session.is_some()
    }

    /// Build the full URL for an endpoint
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.api_url, path)
    }

    /// Make a GET request
    pub async fn get(&self, path: &str) -> Result<Response, ApiError> {
        let mut request = self.client.get(self.url(path));

        if let Some(session) = &self.session {
            request = request.header("Cookie", format!("admin_session={}", session.session_token));
        }

        let response = request.send().await?;
        Ok(response)
    }

    /// Make an authenticated GET request (fails if not authenticated)
    pub async fn get_auth(&self, path: &str) -> Result<Response, ApiError> {
        if !self.is_authenticated() {
            return Err(ApiError::Unauthorized);
        }
        self.get(path).await
    }

    /// Make a POST request with JSON body
    pub async fn post<T: Serialize>(&self, path: &str, body: &T) -> Result<Response, ApiError> {
        let mut request = self.client.post(self.url(path)).json(body);

        if let Some(session) = &self.session {
            request = request.header("Cookie", format!("admin_session={}", session.session_token));
        }

        let response = request.send().await?;
        Ok(response)
    }

    /// Make an authenticated POST request
    pub async fn post_auth<T: Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<Response, ApiError> {
        if !self.is_authenticated() {
            return Err(ApiError::Unauthorized);
        }
        self.post(path, body).await
    }

    /// Make a PUT request with JSON body
    pub async fn put<T: Serialize>(&self, path: &str, body: &T) -> Result<Response, ApiError> {
        let mut request = self.client.put(self.url(path)).json(body);

        if let Some(session) = &self.session {
            request = request.header("Cookie", format!("admin_session={}", session.session_token));
        }

        let response = request.send().await?;
        Ok(response)
    }

    /// Make an authenticated PUT request
    pub async fn put_auth<T: Serialize>(&self, path: &str, body: &T) -> Result<Response, ApiError> {
        if !self.is_authenticated() {
            return Err(ApiError::Unauthorized);
        }
        self.put(path, body).await
    }

    /// Make a DELETE request
    pub async fn delete(&self, path: &str) -> Result<Response, ApiError> {
        let mut request = self.client.delete(self.url(path));

        if let Some(session) = &self.session {
            request = request.header("Cookie", format!("admin_session={}", session.session_token));
        }

        let response = request.send().await?;
        Ok(response)
    }

    /// Make an authenticated DELETE request
    pub async fn delete_auth(&self, path: &str) -> Result<Response, ApiError> {
        if !self.is_authenticated() {
            return Err(ApiError::Unauthorized);
        }
        self.delete(path).await
    }
}

/// Helper to check response and extract error message
pub async fn check_response(response: Response) -> Result<Response, ApiError> {
    let status = response.status();
    if status.is_success() {
        Ok(response)
    } else {
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(ApiError::Http { status, body })
    }
}
