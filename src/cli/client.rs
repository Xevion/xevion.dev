use reqwest::{Client, Response, StatusCode};
use serde::Serialize;

/// API client that authenticates with a long-lived CLI bearer token.
pub struct ApiClient {
    pub api_url: String,
    token: Option<String>,
    client: Client,
}

#[derive(Debug)]
pub enum ApiError {
    /// HTTP request failed
    Request(reqwest::Error),
    /// Server returned an error response
    Http { status: StatusCode, body: String },
    /// Not authenticated
    Unauthorized,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Request(e) => write!(f, "Request failed: {e}"),
            Self::Http { status, body } => {
                write!(f, "HTTP {status}: {body}")
            }
            Self::Unauthorized => write!(f, "Not authenticated. Run 'xevion api login' first."),
        }
    }
}

impl std::error::Error for ApiError {}

impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self {
        Self::Request(e)
    }
}

impl ApiClient {
    /// Create a new API client for `api_url`, optionally authenticated by `token`.
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(api_url: String, token: Option<String>) -> Self {
        Self {
            // Normalize away a trailing slash so `url()` joins cleanly.
            api_url: api_url.trim_end_matches('/').to_string(),
            token,
            client: Client::new(),
        }
    }

    /// Check if we have a token to authenticate with.
    pub const fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }

    /// Build the full URL for an endpoint
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.api_url, path)
    }

    /// Attach the bearer token to a request if we have one.
    fn authed(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(token) = &self.token {
            request.bearer_auth(token)
        } else {
            request
        }
    }

    /// Make a GET request
    pub async fn get(&self, path: &str) -> Result<Response, ApiError> {
        let response = self.authed(self.client.get(self.url(path))).send().await?;
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
        let response = self
            .authed(self.client.post(self.url(path)).json(body))
            .send()
            .await?;
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
        let response = self
            .authed(self.client.put(self.url(path)).json(body))
            .send()
            .await?;
        Ok(response)
    }

    /// Make an authenticated PUT request
    pub async fn put_auth<T: Serialize>(&self, path: &str, body: &T) -> Result<Response, ApiError> {
        if !self.is_authenticated() {
            return Err(ApiError::Unauthorized);
        }
        self.put(path, body).await
    }

    /// Make a PATCH request with JSON body
    pub async fn patch<T: Serialize>(&self, path: &str, body: &T) -> Result<Response, ApiError> {
        let response = self
            .authed(self.client.patch(self.url(path)).json(body))
            .send()
            .await?;
        Ok(response)
    }

    /// Make an authenticated PATCH request
    pub async fn patch_auth<T: Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<Response, ApiError> {
        if !self.is_authenticated() {
            return Err(ApiError::Unauthorized);
        }
        self.patch(path, body).await
    }

    /// Make a DELETE request
    pub async fn delete(&self, path: &str) -> Result<Response, ApiError> {
        let response = self
            .authed(self.client.delete(self.url(path)))
            .send()
            .await?;
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
