//! Public-host resolution for per-domain SSR output.
//!
//! The resolved host becomes part of the ISR cache key (so each public domain
//! gets its own cached SSR variant) and is forwarded downstream to Bun so the
//! rendered HTML can carry a per-domain origin (`og:url`, `<link rel=canonical>`,
//! absolute URLs).
//!
//! `X-Forwarded-Host` is *not* trustworthy as-is: behind Railway's proxy it is
//! attacker-controllable. An ungated value would be a cache-poisoning vector and
//! would let arbitrary hosts multiply cache cardinality without bound. So the
//! incoming host is gated against an allowlist of known public hosts and falls
//! back to a canonical default otherwise.

use axum::http::{HeaderMap, header};

/// Allowlist-gated resolver for the public host a request is served under.
///
/// Configured from the environment (see [`HostConfig::from_env`]). When no
/// allowlist is configured the resolver runs in *permissive* mode — the incoming
/// host is trusted as-is. That is the local-dev posture, where the server isn't
/// publicly exposed and `localhost:<port>` should flow through unchanged.
#[derive(Debug, Clone)]
pub struct HostConfig {
    /// Accepted public hostnames, lowercased with any port stripped.
    allowed: Vec<String>,
    /// Canonical fallback host used when the request host isn't allowed.
    canonical: String,
    /// True when no allowlist is configured (local dev) — trust the request host.
    permissive: bool,
}

impl HostConfig {
    /// Load configuration from the environment.
    ///
    /// - `PUBLIC_HOSTS` — comma-separated allowlist of public hostnames
    ///   (e.g. `xevion.dev,walters.to`). Empty/unset enables permissive mode.
    /// - `CANONICAL_HOST` — the fallback host for requests that aren't allowed.
    ///   Defaults to the first entry of `PUBLIC_HOSTS`, or `localhost` if neither
    ///   is set.
    pub fn from_env() -> Self {
        let allowed: Vec<String> = std::env::var("PUBLIC_HOSTS")
            .unwrap_or_default()
            .split(',')
            .filter_map(|raw| {
                let host = normalize_hostname(raw);
                (!host.is_empty()).then_some(host)
            })
            .collect();

        let canonical = std::env::var("CANONICAL_HOST")
            .ok()
            .map(|v| normalize_hostname(&v))
            .filter(|h| !h.is_empty())
            .or_else(|| allowed.first().cloned())
            .unwrap_or_else(|| "localhost".to_string());

        let permissive = allowed.is_empty();

        Self {
            allowed,
            canonical,
            permissive,
        }
    }

    /// Construct a config directly (used in tests).
    #[cfg(test)]
    pub fn new(allowed: &[&str], canonical: impl Into<String>) -> Self {
        let allowed: Vec<String> = allowed.iter().map(|h| normalize_hostname(h)).collect();
        Self {
            permissive: allowed.is_empty(),
            allowed,
            canonical: normalize_hostname(&canonical.into()),
        }
    }

    /// The canonical fallback host.
    pub fn canonical(&self) -> &str {
        &self.canonical
    }

    /// The public URL scheme. Allowlisted hosts are always served over HTTPS
    /// (TLS terminates at the edge); permissive dev mode stays on HTTP so
    /// `localhost` origins resolve to `http://localhost:<port>`.
    pub const fn scheme(&self) -> &'static str {
        if self.permissive { "http" } else { "https" }
    }

    /// Resolve the trusted public host for a request from its headers.
    ///
    /// Prefers `X-Forwarded-Host` (set by the proxy), falling back to `Host`.
    /// In permissive mode the candidate authority is returned verbatim (lowercased,
    /// port preserved) so local dev keeps its `localhost:<port>` origin. Otherwise
    /// the hostname is matched against the allowlist — a match returns the bare
    /// hostname, anything else returns the canonical default.
    pub fn resolve(&self, headers: &HeaderMap) -> String {
        let candidate = candidate_authority(headers);

        if self.permissive {
            return candidate.map_or_else(|| self.canonical.clone(), |c| c.to_ascii_lowercase());
        }

        match candidate {
            Some(authority) => {
                let hostname = normalize_hostname(&authority);
                if self.allowed.iter().any(|h| h == &hostname) {
                    hostname
                } else {
                    self.canonical.clone()
                }
            }
            None => self.canonical.clone(),
        }
    }
}

/// Extract the first authority from `X-Forwarded-Host` (preferred) or `Host`.
///
/// `X-Forwarded-Host` may carry a proxy chain (`a.example, b.example`); only the
/// first (client-facing) value is meaningful.
fn candidate_authority(headers: &HeaderMap) -> Option<String> {
    let raw = headers
        .get("x-forwarded-host")
        .or_else(|| headers.get(header::HOST))
        .and_then(|v| v.to_str().ok())?;

    let first = raw.split(',').next()?.trim();
    (!first.is_empty()).then(|| first.to_string())
}

/// Lowercase an authority and strip any `:port` suffix, yielding a bare hostname.
fn normalize_hostname(authority: &str) -> String {
    let trimmed = authority.trim();
    let host = trimmed.split(':').next().unwrap_or(trimmed);
    host.to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers_with(name: &str, value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        let header_name = axum::http::HeaderName::from_bytes(name.as_bytes()).unwrap();
        headers.insert(header_name, value.parse().unwrap());
        headers
    }

    #[test]
    fn allowed_host_passes_through() {
        let config = HostConfig::new(&["xevion.dev", "walters.to"], "xevion.dev");
        let headers = headers_with("x-forwarded-host", "walters.to");
        assert_eq!(config.resolve(&headers), "walters.to");
    }

    #[test]
    fn disallowed_host_falls_back_to_canonical() {
        let config = HostConfig::new(&["xevion.dev"], "xevion.dev");
        let headers = headers_with("x-forwarded-host", "evil.example.com");
        assert_eq!(config.resolve(&headers), "xevion.dev");
    }

    #[test]
    fn missing_host_uses_canonical() {
        let config = HostConfig::new(&["xevion.dev"], "xevion.dev");
        assert_eq!(config.resolve(&HeaderMap::new()), "xevion.dev");
    }

    #[test]
    fn forwarded_host_takes_precedence_over_host() {
        let config = HostConfig::new(&["xevion.dev", "walters.to"], "xevion.dev");
        let mut headers = headers_with("x-forwarded-host", "walters.to");
        headers.insert(header::HOST, "internal.railway".parse().unwrap());
        assert_eq!(config.resolve(&headers), "walters.to");
    }

    #[test]
    fn port_is_stripped_before_matching() {
        let config = HostConfig::new(&["xevion.dev"], "xevion.dev");
        let headers = headers_with("x-forwarded-host", "xevion.dev:443");
        assert_eq!(config.resolve(&headers), "xevion.dev");
    }

    #[test]
    fn proxy_chain_uses_first_value() {
        let config = HostConfig::new(&["walters.to"], "xevion.dev");
        let headers = headers_with("x-forwarded-host", "walters.to, internal.railway");
        assert_eq!(config.resolve(&headers), "walters.to");
    }

    #[test]
    fn case_is_normalized() {
        let config = HostConfig::new(&["xevion.dev"], "xevion.dev");
        let headers = headers_with("x-forwarded-host", "Xevion.DEV");
        assert_eq!(config.resolve(&headers), "xevion.dev");
    }

    #[test]
    fn permissive_mode_trusts_incoming_host_with_port() {
        let config = HostConfig::new(&[], "localhost");
        let headers = headers_with(header::HOST.as_str(), "localhost:10237");
        assert_eq!(config.resolve(&headers), "localhost:10237");
    }

    #[test]
    fn permissive_mode_without_host_uses_canonical() {
        let config = HostConfig::new(&[], "localhost");
        assert_eq!(config.resolve(&HeaderMap::new()), "localhost");
    }

    #[test]
    fn scheme_is_https_when_allowlisted_and_http_in_permissive_mode() {
        assert_eq!(
            HostConfig::new(&["xevion.dev"], "xevion.dev").scheme(),
            "https"
        );
        assert_eq!(HostConfig::new(&[], "localhost").scheme(), "http");
    }
}
