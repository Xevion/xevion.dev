use reqwest::{Client, Response, StatusCode};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::cli::error::{CliError, DecodeSnafu};

/// API client that authenticates with a long-lived CLI bearer token.
///
/// A client is only ever built from a resolved, authorized target (see
/// [`authed_client`](crate::cli::api::authed_client)) or, for the public read
/// endpoints, an explicitly token-less one. Either way the server is the single
/// source of truth for authorization — there is no second client-side guard.
pub struct ApiClient {
    pub api_url: String,
    token: Option<String>,
    client: Client,
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

    /// Build the full URL for an endpoint.
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

    /// Send a request, mapping transport failures to a connect/request diagnostic
    /// that names the URL.
    async fn send(
        &self,
        request: reqwest::RequestBuilder,
        url: String,
    ) -> Result<Response, CliError> {
        request
            .send()
            .await
            .map_err(|e| CliError::from_send(url, e))
    }

    pub async fn get(&self, path: &str) -> Result<Response, CliError> {
        let url = self.url(path);
        self.send(self.authed(self.client.get(&url)), url).await
    }

    pub async fn post<T: Serialize>(&self, path: &str, body: &T) -> Result<Response, CliError> {
        let url = self.url(path);
        self.send(self.authed(self.client.post(&url).json(body)), url)
            .await
    }

    pub async fn put<T: Serialize>(&self, path: &str, body: &T) -> Result<Response, CliError> {
        let url = self.url(path);
        self.send(self.authed(self.client.put(&url).json(body)), url)
            .await
    }

    pub async fn patch<T: Serialize>(&self, path: &str, body: &T) -> Result<Response, CliError> {
        let url = self.url(path);
        self.send(self.authed(self.client.patch(&url).json(body)), url)
            .await
    }

    pub async fn delete(&self, path: &str) -> Result<Response, CliError> {
        let url = self.url(path);
        self.send(self.authed(self.client.delete(&url)), url).await
    }
}

/// The API's standard error envelope (see `docs/STYLE.md`).
#[derive(Deserialize)]
struct ApiErrorBody {
    error: Option<String>,
    code: Option<String>,
}

/// Turn a non-2xx response into a diagnostic. A 401 maps to
/// [`CliError::Unauthorized`]; otherwise the server's `{ error, code }` body is
/// surfaced, falling back to the raw text when it isn't the standard envelope.
pub async fn check_response(response: Response) -> Result<Response, CliError> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    if status == StatusCode::UNAUTHORIZED {
        return Err(CliError::Unauthorized);
    }

    let body = response.text().await.unwrap_or_default();
    let (code, message) = match serde_json::from_str::<ApiErrorBody>(&body) {
        Ok(parsed) => (
            parsed.code,
            parsed.error.unwrap_or_else(|| fallback_message(&body)),
        ),
        Err(_) => (None, fallback_message(&body)),
    };

    Err(CliError::Http {
        status,
        code,
        message,
    })
}

fn fallback_message(body: &str) -> String {
    if body.trim().is_empty() {
        "(empty response body)".to_string()
    } else {
        body.to_string()
    }
}

/// Decode a JSON response body, mapping decode failures to a diagnostic.
pub async fn json<T: serde::de::DeserializeOwned>(response: Response) -> Result<T, CliError> {
    response.json().await.context(DecodeSnafu)
}
