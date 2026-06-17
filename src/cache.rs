//! ISR (Incremental Static Regeneration) cache implementation
//!
//! Provides in-memory caching for SSR pages with:
//! - TTL-based expiration
//! - Stale-while-revalidate pattern
//! - Singleflight (via moka's built-in coalescing)
//! - Multi-encoding compressed storage (lazy)
//! - On-demand invalidation

use axum::http::{HeaderMap, StatusCode};
use dashmap::DashSet;
use moka::future::Cache;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::encoding::{
    COMPRESSION_MIN_SIZE, ContentEncoding, compress_brotli, compress_gzip, compress_zstd,
};

/// Cached response data with lazy compressed variants
#[derive(Clone)]
pub struct CachedResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    /// Original uncompressed body
    pub body: axum::body::Bytes,
    /// Compressed variants (lazily populated on first request per encoding)
    compressed: Arc<parking_lot::RwLock<HashMap<ContentEncoding, axum::body::Bytes>>>,
    pub cached_at: Instant,
}

impl CachedResponse {
    pub fn new(status: StatusCode, headers: HeaderMap, body: axum::body::Bytes) -> Self {
        Self {
            status,
            headers,
            body,
            compressed: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            cached_at: Instant::now(),
        }
    }

    /// Get body for a specific encoding, compressing on-demand if needed
    ///
    /// Returns (`body_bytes`, `actual_encoding`). The actual encoding may differ from
    /// requested if the body is too small or compression doesn't help.
    pub fn get_body(&self, encoding: ContentEncoding) -> (axum::body::Bytes, ContentEncoding) {
        // Identity encoding or small body - return uncompressed
        if encoding == ContentEncoding::Identity || self.body.len() < COMPRESSION_MIN_SIZE as usize
        {
            return (self.body.clone(), ContentEncoding::Identity);
        }

        // Check if we already have this encoding cached
        {
            let cache = self.compressed.read();
            if let Some(compressed) = cache.get(&encoding) {
                return (compressed.clone(), encoding);
            }
        }

        // Compress on-demand
        let compressed_bytes = match encoding {
            ContentEncoding::Zstd => compress_zstd(&self.body),
            ContentEncoding::Brotli => compress_brotli(&self.body),
            ContentEncoding::Gzip => compress_gzip(&self.body),
            ContentEncoding::Identity => unreachable!(),
        };

        // Only cache if compression actually helped
        if let Some(compressed) = compressed_bytes
            && compressed.len() < self.body.len()
        {
            let bytes = axum::body::Bytes::from(compressed);
            self.compressed.write().insert(encoding, bytes.clone());
            return (bytes, encoding);
        }

        // Compression didn't help or failed, return uncompressed
        (self.body.clone(), ContentEncoding::Identity)
    }

    /// Check if this response is still fresh (within `fresh_duration`)
    pub fn is_fresh(&self, fresh_duration: Duration) -> bool {
        self.cached_at.elapsed() < fresh_duration
    }

    /// Check if this response is stale but still usable (within `stale_duration`)
    pub fn is_stale_but_usable(&self, fresh_duration: Duration, stale_duration: Duration) -> bool {
        let age = self.cached_at.elapsed();
        age >= fresh_duration && age < stale_duration
    }

    /// Get the age of this cached response
    pub fn age(&self) -> Duration {
        self.cached_at.elapsed()
    }
}

/// Configuration for the ISR cache
#[derive(Debug, Clone)]
pub struct IsrCacheConfig {
    /// Maximum number of cached entries
    pub max_entries: u64,
    /// Duration a response is considered fresh (served without refresh)
    pub fresh_duration: Duration,
    /// Total duration before entry is evicted (stale responses served during refresh)
    pub stale_duration: Duration,
    /// Whether caching is enabled
    pub enabled: bool,
}

impl Default for IsrCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            fresh_duration: Duration::from_mins(1),
            stale_duration: Duration::from_mins(5),
            enabled: true,
        }
    }
}

impl IsrCacheConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let max_entries = std::env::var("ISR_CACHE_MAX_ENTRIES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1000);

        let fresh_sec = std::env::var("ISR_CACHE_FRESH_SEC")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);

        let stale_sec = std::env::var("ISR_CACHE_STALE_SEC")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(300);

        let enabled = std::env::var("ISR_CACHE_ENABLED").map_or(true, |v| v != "false" && v != "0");

        Self {
            max_entries,
            fresh_duration: Duration::from_secs(fresh_sec),
            stale_duration: Duration::from_secs(stale_sec),
            enabled,
        }
    }
}

/// ISR cache for SSR page responses
pub struct IsrCache {
    cache: Cache<String, Arc<CachedResponse>>,
    /// Tracks paths currently being refreshed in background
    refreshing: DashSet<String>,
    pub config: IsrCacheConfig,
}

impl IsrCache {
    /// Create a new ISR cache with the given configuration
    pub fn new(config: IsrCacheConfig) -> Self {
        let cache = Cache::builder()
            .max_capacity(config.max_entries)
            // Use stale_duration as TTL - we handle fresh/stale logic ourselves
            .time_to_live(config.stale_duration)
            .name("isr_cache")
            .build();

        Self {
            cache,
            refreshing: DashSet::new(),
            config,
        }
    }

    /// Get a cached response if it exists
    pub async fn get(&self, path: &str) -> Option<Arc<CachedResponse>> {
        if !self.config.enabled {
            return None;
        }
        self.cache.get(path).await
    }

    /// Insert a response into the cache
    pub async fn insert(&self, path: String, response: CachedResponse) {
        if !self.config.enabled {
            return;
        }
        self.cache.insert(path, Arc::new(response)).await;
    }

    /// Mark a path as being refreshed. Returns true if it wasn't already refreshing.
    pub fn start_refresh(&self, path: &str) -> bool {
        self.refreshing.insert(path.to_string())
    }

    /// Mark a path refresh as complete
    pub fn end_refresh(&self, path: &str) {
        self.refreshing.remove(path);
    }

    /// Invalidate a cached request target across every public host.
    ///
    /// Keys are `{host}{path}[?query]` (see [`cache_key`]); the host never
    /// contains `/`, so the first `/` begins the request target. Callers pass a
    /// bare target (e.g. `/` or `/projects/foo`), and every host's variant of
    /// that exact target is dropped.
    pub async fn invalidate(&self, path: &str) {
        let mut invalidated = 0u32;
        for (key, _) in &self.cache {
            if let Some(idx) = key.find('/')
                && &key[idx..] == path
            {
                self.cache.invalidate(key.as_str()).await;
                invalidated += 1;
            }
        }
        tracing::debug!(path = %path, invalidated, "Cache entries invalidated");
    }
}

/// Excludes admin, API, internal, and static asset paths.
pub fn is_cacheable_path(path: &str) -> bool {
    !path.starts_with("/admin")
        && !path.starts_with("/api/")
        && !path.starts_with("/internal/")
        && !path.starts_with("/_app/")
        && !path.starts_with("/.")
}

/// Build the downstream request target (path + query) sent to Bun.
pub fn request_target(path: &str, query: Option<&str>) -> String {
    match query {
        Some(q) if !q.is_empty() => format!("{path}?{q}"),
        _ => path.to_string(),
    }
}

/// Build the ISR cache key for a validated public host and request target.
///
/// The key is `{host}{target}` — since a hostname never contains `/` and a
/// request target always starts with `/`, the first `/` unambiguously separates
/// the two. Keying by host gives each public domain its own cached SSR variant;
/// the host allowlist (see [`crate::host`]) keeps the cardinality finite.
pub fn cache_key(host: &str, path: &str, query: Option<&str>) -> String {
    cache_key_for_target(host, &request_target(path, query))
}

/// Build the ISR cache key from an already-computed request target.
pub fn cache_key_for_target(host: &str, target: &str) -> String {
    format!("{host}{target}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_cacheable_path() {
        // Should cache
        assert!(is_cacheable_path("/"));
        assert!(is_cacheable_path("/some-page"));
        assert!(is_cacheable_path("/pgp"));

        // Should not cache
        assert!(!is_cacheable_path("/admin"));
        assert!(!is_cacheable_path("/admin/projects"));
        assert!(!is_cacheable_path("/api/projects"));
        assert!(!is_cacheable_path("/internal/health"));
        assert!(!is_cacheable_path("/_app/immutable/foo.js"));
    }

    #[test]
    fn test_request_target() {
        assert_eq!(request_target("/", None), "/");
        assert_eq!(request_target("/", Some("")), "/");
        assert_eq!(request_target("/", Some("tag=rust")), "/?tag=rust");
        assert_eq!(request_target("/some-page", None), "/some-page");
    }

    #[test]
    fn test_cache_key() {
        assert_eq!(cache_key("xevion.dev", "/", None), "xevion.dev/");
        assert_eq!(cache_key("xevion.dev", "/", Some("")), "xevion.dev/");
        assert_eq!(
            cache_key("xevion.dev", "/", Some("tag=rust")),
            "xevion.dev/?tag=rust"
        );
        assert_eq!(
            cache_key("walters.to", "/some-page", None),
            "walters.to/some-page"
        );
        // Same target under different hosts yields distinct keys.
        assert_ne!(
            cache_key("xevion.dev", "/", None),
            cache_key("walters.to", "/", None)
        );
    }

    #[tokio::test]
    async fn invalidate_clears_target_across_hosts() {
        let cache = IsrCache::new(IsrCacheConfig::default());
        let body = axum::body::Bytes::from_static(b"x");
        let resp = || CachedResponse::new(StatusCode::OK, HeaderMap::new(), body.clone());

        cache
            .insert(cache_key("xevion.dev", "/", None), resp())
            .await;
        cache
            .insert(cache_key("walters.to", "/", None), resp())
            .await;
        cache
            .insert(cache_key("xevion.dev", "/projects/foo", None), resp())
            .await;

        cache.invalidate("/").await;

        // Both hosts' homepage variants are gone.
        assert!(
            cache
                .get(&cache_key("xevion.dev", "/", None))
                .await
                .is_none()
        );
        assert!(
            cache
                .get(&cache_key("walters.to", "/", None))
                .await
                .is_none()
        );
        // A different target is untouched.
        assert!(
            cache
                .get(&cache_key("xevion.dev", "/projects/foo", None))
                .await
                .is_some()
        );
    }

    #[tokio::test]
    async fn test_cached_response_freshness() {
        let response = CachedResponse::new(
            StatusCode::OK,
            HeaderMap::new(),
            axum::body::Bytes::from_static(b"test"),
        );

        let fresh = Duration::from_millis(100);
        let stale = Duration::from_millis(200);

        // Should be fresh immediately
        assert!(response.is_fresh(fresh));
        assert!(!response.is_stale_but_usable(fresh, stale));

        // Wait a bit
        tokio::time::sleep(Duration::from_millis(110)).await;

        // Should be stale but usable
        assert!(!response.is_fresh(fresh));
        assert!(response.is_stale_but_usable(fresh, stale));

        // Wait more
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Should be neither fresh nor usable
        assert!(!response.is_fresh(fresh));
        assert!(!response.is_stale_but_usable(fresh, stale));
    }
}
