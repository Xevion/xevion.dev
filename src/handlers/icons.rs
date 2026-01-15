//! Icon serving handler with caching
//!
//! Serves SVG icons with aggressive HTTP caching. On cache miss, proxies to
//! Bun's icon API and caches the result.

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use std::{collections::HashMap, sync::Arc};

use crate::{proxy, state::AppState};

/// Response from Bun's icon API
#[derive(serde::Deserialize)]
struct IconApiResponse {
    svg: String,
}

/// Handle icon requests - serves cached SVG or proxies to Bun
///
/// Route: GET /api/icons/{*path}
/// - For `{collection}/{name}.svg` paths: serve cached SVG with aggressive caching
/// - For all other paths: proxy to Bun's icon API (search, collections, JSON responses)
pub async fn serve_icon_handler(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    // Check if this is a cacheable SVG request: {collection}/{name}.svg
    if let Some((collection, name)) = parse_svg_path(&path) {
        return serve_cached_svg(state, collection, name).await;
    }

    // Not an SVG request - proxy to Bun (with query string for search)
    proxy_to_bun_icons(&path, &query, state).await
}

/// Parse path to extract collection and name if it matches {collection}/{name}.svg
fn parse_svg_path(path: &str) -> Option<(&str, &str)> {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() == 2 {
        let collection = parts[0];
        let filename = parts[1];
        if let Some(name) = filename.strip_suffix(".svg")
            && !collection.is_empty()
            && !name.is_empty()
        {
            return Some((collection, name));
        }
    }
    None
}

/// Serve an SVG icon with caching
async fn serve_cached_svg(state: Arc<AppState>, collection: &str, name: &str) -> Response {
    let cache_key = format!("{collection}:{name}");

    // Check cache first
    if let Some(svg) = state.icon_cache.get(&cache_key).await {
        tracing::trace!(
            collection = %collection,
            name = %name,
            "Icon cache hit"
        );
        return svg_response(&svg);
    }

    // Cache miss - fetch from Bun's icon API
    let bun_path = format!("/api/icons/{collection}/{name}");
    let forward_headers = HeaderMap::new();

    match proxy::proxy_to_bun(&bun_path, state.clone(), forward_headers).await {
        Ok((status, _headers, body)) if status.is_success() => {
            // Parse JSON response from Bun
            match serde_json::from_slice::<IconApiResponse>(&body) {
                Ok(data) => {
                    // Cache the SVG
                    state.icon_cache.insert(cache_key, data.svg.clone()).await;

                    tracing::debug!(
                        collection = %collection,
                        name = %name,
                        "Icon cached"
                    );

                    svg_response(&data.svg)
                }
                Err(e) => {
                    tracing::error!(
                        error = %e,
                        collection = %collection,
                        name = %name,
                        "Failed to parse icon response"
                    );
                    (StatusCode::INTERNAL_SERVER_ERROR, "Invalid icon data").into_response()
                }
            }
        }
        Ok((status, _, _)) => {
            tracing::debug!(
                status = %status,
                collection = %collection,
                name = %name,
                "Icon not found"
            );
            (status, "Icon not found").into_response()
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                collection = %collection,
                name = %name,
                "Failed to proxy icon request"
            );
            (StatusCode::BAD_GATEWAY, "Icon service unavailable").into_response()
        }
    }
}

/// Proxy non-SVG icon requests to Bun
async fn proxy_to_bun_icons(
    path: &str,
    query: &HashMap<String, String>,
    state: Arc<AppState>,
) -> Response {
    let query_string = if query.is_empty() {
        String::new()
    } else {
        let qs: Vec<String> = query
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect();
        format!("?{}", qs.join("&"))
    };
    let bun_path = format!("/api/icons/{path}{query_string}");
    let forward_headers = HeaderMap::new();

    match proxy::proxy_to_bun(&bun_path, state, forward_headers).await {
        Ok((status, headers, body)) => (status, headers, body).into_response(),
        Err(e) => {
            tracing::error!(error = %e, path = %path, "Failed to proxy icon request");
            (StatusCode::BAD_GATEWAY, "Icon service unavailable").into_response()
        }
    }
}

/// Build SVG response with aggressive cache headers
fn svg_response(svg: &str) -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("image/svg+xml"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000, immutable"),
    );

    (StatusCode::OK, headers, svg.to_string()).into_response()
}
