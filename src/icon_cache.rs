//! Icon cache for serving SVG icons with aggressive HTTP caching
//!
//! Icons are immutable for a given identifier (e.g., "lucide:home" always returns
//! the same SVG), so we can cache them with long TTLs and immutable HTTP headers.

use moka::future::Cache;
use std::{sync::Arc, time::Duration};

/// Cache for rendered SVG icons
pub struct IconCache {
    cache: Cache<String, Arc<String>>,
}

impl IconCache {
    /// Create a new icon cache
    ///
    /// Config: 10,000 max entries, 24-hour TTL
    pub fn new() -> Self {
        let cache = Cache::builder()
            .max_capacity(10_000)
            .time_to_live(Duration::from_secs(86400)) // 24 hours
            .name("icon_cache")
            .build();

        Self { cache }
    }

    /// Get a cached SVG if it exists
    pub async fn get(&self, identifier: &str) -> Option<Arc<String>> {
        self.cache.get(identifier).await
    }

    /// Insert an SVG into the cache
    pub async fn insert(&self, identifier: String, svg: String) {
        self.cache.insert(identifier, Arc::new(svg)).await;
    }
}

impl Default for IconCache {
    fn default() -> Self {
        Self::new()
    }
}
