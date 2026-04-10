use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Failed to build reqwest client: {0}")]
    BuildError(#[from] reqwest::Error),

    #[error("Invalid downstream URL: {0}")]
    InvalidUrl(String),
}

#[derive(Clone)]
pub struct HttpClient {
    client: reqwest::Client,
    target: TargetUrl,
}

#[derive(Debug, Clone)]
enum TargetUrl {
    Tcp(String),   // Base URL like "http://localhost:5173"
    Unix(PathBuf), // Socket path like "/tmp/bun.sock"
}

impl TargetUrl {
    fn build_url(&self, path: &str) -> String {
        match self {
            Self::Tcp(base) => format!("{base}{path}"),
            Self::Unix(_) => format!("http://localhost{path}"),
        }
    }
}

impl HttpClient {
    /// Create a new `HttpClient` from a downstream URL
    ///
    /// Accepts:
    /// - TCP: "<http://localhost:5173>", "<https://example.com>"
    /// - Unix: "/tmp/bun.sock", "./relative.sock"
    pub fn new(downstream: &str) -> Result<Self, ClientError> {
        let target = if downstream.starts_with('/') || downstream.starts_with("./") {
            TargetUrl::Unix(PathBuf::from(downstream))
        } else if downstream.starts_with("http://") || downstream.starts_with("https://") {
            TargetUrl::Tcp(downstream.to_string())
        } else {
            return Err(ClientError::InvalidUrl(downstream.to_string()));
        };

        tracing::debug!(
            target = ?target,
            downstream = %downstream,
            "Creating HTTP client"
        );

        let client = match &target {
            TargetUrl::Unix(path) => reqwest::Client::builder()
                .pool_max_idle_per_host(8)
                .pool_idle_timeout(Duration::from_secs(600))
                .timeout(Duration::from_secs(5))
                .connect_timeout(Duration::from_secs(3))
                .redirect(reqwest::redirect::Policy::none())
                .unix_socket(path.clone())
                .build()?,
            TargetUrl::Tcp(_) => reqwest::Client::builder()
                .pool_max_idle_per_host(8)
                .pool_idle_timeout(Duration::from_secs(600))
                .tcp_keepalive(Some(Duration::from_secs(60)))
                .timeout(Duration::from_secs(5))
                .connect_timeout(Duration::from_secs(3))
                .redirect(reqwest::redirect::Policy::none())
                .build()?,
        };

        Ok(Self { client, target })
    }

    /// Build a full URL from a path
    ///
    /// Examples:
    /// - TCP target "<http://localhost:5173>" + "/api/health" → "<http://localhost:5173/api/health>"
    /// - Unix target "/tmp/bun.sock" + "/api/health" → "<http://localhost/api/health>"
    fn build_url(&self, path: &str) -> String {
        self.target.build_url(path)
    }

    pub fn get(&self, path: &str) -> reqwest::RequestBuilder {
        self.client.get(self.build_url(path))
    }

    pub fn post(&self, path: &str) -> reqwest::RequestBuilder {
        self.client.post(self.build_url(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_url_construction() {
        let target = TargetUrl::Tcp("http://localhost:5173".to_string());
        assert_eq!(
            target.build_url("/api/health"),
            "http://localhost:5173/api/health"
        );
        assert_eq!(
            target.build_url("/path?query=1"),
            "http://localhost:5173/path?query=1"
        );
    }

    #[test]
    fn test_unix_url_construction() {
        let target = TargetUrl::Unix(PathBuf::from("/tmp/bun.sock"));
        assert_eq!(
            target.build_url("/api/health"),
            "http://localhost/api/health"
        );
        assert_eq!(
            target.build_url("/path?query=1"),
            "http://localhost/path?query=1"
        );
    }

    #[test]
    fn test_https_url_construction() {
        let target = TargetUrl::Tcp("https://example.com".to_string());
        assert_eq!(
            target.build_url("/api/test"),
            "https://example.com/api/test"
        );
    }

    #[test]
    fn test_invalid_url() {
        let result = HttpClient::new("not-a-valid-url");
        assert!(result.is_err());
    }
}
