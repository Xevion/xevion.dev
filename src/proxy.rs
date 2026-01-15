use axum::{
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use std::{net::SocketAddr, sync::Arc, time::Duration};

use crate::{
    assets,
    cache::{self, CachedResponse},
    db,
    encoding::negotiate_encoding,
    state::{AppState, ProxyError},
    tarpit::{self, TarpitState},
    utils,
};

/// ISR handler - serves pages through Bun SSR with caching and session validation
pub async fn isr_handler(State(state): State<Arc<AppState>>, req: Request) -> Response {
    let method = req.method().clone();
    let uri = req.uri();
    let path = uri.path();
    let query = uri.query();
    let request_headers = req.headers().clone();

    // Redirect trailing slashes to non-trailing (except root)
    if path.len() > 1 && path.ends_with('/') {
        let normalized = path.trim_end_matches('/');
        let redirect_uri = match query {
            Some(q) => format!("{normalized}?{q}"),
            None => normalized.to_string(),
        };
        return axum::response::Redirect::permanent(&redirect_uri).into_response();
    }

    // Redirect .html extensions to clean URLs (skip static assets)
    if path.ends_with(".html") && !path.starts_with("/_app/") {
        let clean_path = if path == "/index.html" {
            "/".to_string()
        } else {
            path.strip_suffix(".html").unwrap().to_string()
        };
        let redirect_uri = match query {
            Some(q) => format!("{clean_path}?{q}"),
            None => clean_path,
        };
        return axum::response::Redirect::permanent(&redirect_uri).into_response();
    }

    if method != axum::http::Method::GET && method != axum::http::Method::HEAD {
        tracing::warn!(method = %method, path = %path, "Non-GET/HEAD request to non-API route");

        if utils::accepts_html(req.headers()) {
            return utils::serve_error_page(StatusCode::METHOD_NOT_ALLOWED);
        }

        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::ALLOW,
            axum::http::HeaderValue::from_static("GET, HEAD, OPTIONS"),
        );
        return (
            StatusCode::METHOD_NOT_ALLOWED,
            headers,
            "Method not allowed",
        )
            .into_response();
    }

    let is_head = method == axum::http::Method::HEAD;

    if path.starts_with("/api/") {
        tracing::error!(path = %path, "API request reached ISR handler - routing bug");
        return (StatusCode::INTERNAL_SERVER_ERROR, "Internal routing error").into_response();
    }

    // Block internal routes from external access
    if path.starts_with("/internal/") {
        tracing::warn!(path = %path, "Attempted access to internal route");

        if utils::accepts_html(req.headers()) {
            return utils::serve_error_page(StatusCode::NOT_FOUND);
        }

        return (StatusCode::NOT_FOUND, "Not found").into_response();
    }

    // Check if this is a static asset that exists in embedded CLIENT_ASSETS
    if utils::is_static_asset(path)
        && let Some(response) = assets::try_serve_embedded_asset_with_encoding(path, req.headers())
    {
        return response;
    }
    // If not found in embedded assets, continue to proxy (might be in Bun's static dir)

    // Check if this is a prerendered page
    if let Some(response) = assets::try_serve_prerendered_page(path) {
        tracing::debug!(path = %path, "Serving prerendered page");
        return response;
    }

    let path_with_query = cache::cache_key(path, query);

    // Build trusted headers to forward to downstream
    let mut forward_headers = HeaderMap::new();
    let mut is_authenticated = false;

    // Forward request ID to Bun (set by RequestIdLayer)
    if let Some(request_id) = req.extensions().get::<crate::middleware::RequestId>()
        && let Ok(header_value) = axum::http::HeaderValue::from_str(&request_id.0)
    {
        forward_headers.insert("x-request-id", header_value);
    }

    // SECURITY: Strip any X-Session-User header from incoming request to prevent spoofing

    // Extract and validate session from cookie
    if let Some(cookie_header) = req.headers().get(axum::http::header::COOKIE)
        && let Ok(cookie_str) = cookie_header.to_str()
    {
        // Parse cookies manually to find admin_session
        for cookie_pair in cookie_str.split(';') {
            let cookie_pair = cookie_pair.trim();
            if let Some((name, value)) = cookie_pair.split_once('=')
                && name == "admin_session"
            {
                // Found session cookie, validate it
                if let Ok(session_id) = ulid::Ulid::from_string(value)
                    && let Some(session) = state.session_manager.validate_session(session_id)
                {
                    // Session is valid - add trusted header
                    if let Ok(username_value) = axum::http::HeaderValue::from_str(&session.username)
                    {
                        forward_headers.insert("x-session-user", username_value);
                        is_authenticated = true;
                    }
                }
                break;
            }
        }
    }

    // Determine if this request can use the cache
    // Skip cache for authenticated requests (they see different content)
    let use_cache = !is_authenticated && cache::is_cacheable_path(path);

    // Try to serve from cache for public requests
    if use_cache && let Some(cached) = state.isr_cache.get(&path_with_query).await {
        let fresh_duration = state.isr_cache.config.fresh_duration;
        let stale_duration = state.isr_cache.config.stale_duration;

        if cached.is_fresh(fresh_duration) {
            // Fresh cache hit - serve immediately
            let age_ms = cached.age().as_millis() as u64;
            tracing::debug!(cache = "hit", age_ms, "ISR cache hit (fresh)");

            return serve_cached_response(&cached, &request_headers, is_head);
        } else if cached.is_stale_but_usable(fresh_duration, stale_duration) {
            // Stale cache hit - serve immediately and refresh in background
            let age_ms = cached.age().as_millis() as u64;
            tracing::debug!(cache = "stale", age_ms, "ISR cache hit (stale, refreshing)");

            // Spawn background refresh if not already refreshing
            if state.isr_cache.start_refresh(&path_with_query) {
                let state_clone = state.clone();
                let path_clone = path_with_query.clone();
                tokio::spawn(async move {
                    refresh_cache_entry(state_clone, path_clone).await;
                });
            }

            return serve_cached_response(&cached, &request_headers, is_head);
        }
        // Cache entry is too old - fall through to fetch
    }

    // Cache miss or non-cacheable - fetch from Bun
    let start = std::time::Instant::now();

    match proxy_to_bun(&path_with_query, state.clone(), forward_headers).await {
        Ok((status, headers, body)) => {
            let duration_ms = start.elapsed().as_millis() as u64;

            // Cache successful responses for public requests
            if use_cache && status.is_success() {
                let cached_response = CachedResponse::new(status, headers.clone(), body.clone());
                state
                    .isr_cache
                    .insert(path_with_query.clone(), cached_response)
                    .await;
                tracing::debug!(
                    cache = "miss",
                    status = status.as_u16(),
                    duration_ms,
                    "ISR request (cached)"
                );
            } else {
                log_isr_request(path, status, duration_ms, "bypass");
            }

            // Intercept error responses for HTML requests
            if (status.is_client_error() || status.is_server_error())
                && utils::accepts_html(req.headers())
            {
                return utils::serve_error_page(status);
            }

            if is_head {
                (status, headers).into_response()
            } else {
                (status, headers, body).into_response()
            }
        }
        Err(err) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            tracing::error!(
                error = %err,
                path = %path_with_query,
                duration_ms,
                "Failed to proxy to Bun"
            );

            // Serve 502 error page instead of plaintext
            if utils::accepts_html(req.headers()) {
                return utils::serve_error_page(StatusCode::BAD_GATEWAY);
            }

            (
                StatusCode::BAD_GATEWAY,
                format!("Failed to render page: {err}"),
            )
                .into_response()
        }
    }
}

/// Serve a cached response with content encoding negotiation
fn serve_cached_response(
    cached: &CachedResponse,
    request_headers: &HeaderMap,
    is_head: bool,
) -> Response {
    // Negotiate encoding based on Accept-Encoding
    let desired_encoding = negotiate_encoding(request_headers);
    let (body, actual_encoding) = cached.get_body(desired_encoding);

    let mut headers = cached.headers.clone();

    // Add Content-Encoding header if compressed
    if let Some(encoding_value) = actual_encoding.header_value() {
        headers.insert(header::CONTENT_ENCODING, encoding_value);
    }

    // Add Vary header for caching
    headers.insert(header::VARY, HeaderValue::from_static("Accept-Encoding"));

    // Update Content-Length for compressed body
    if let Ok(len) = HeaderValue::from_str(&body.len().to_string()) {
        headers.insert(header::CONTENT_LENGTH, len);
    }

    if is_head {
        (cached.status, headers).into_response()
    } else {
        (cached.status, headers, body).into_response()
    }
}

/// Background task to refresh a stale cache entry
async fn refresh_cache_entry(state: Arc<AppState>, cache_key: String) {
    // No auth headers for background refresh (public content only)
    let forward_headers = HeaderMap::new();

    match proxy_to_bun(&cache_key, state.clone(), forward_headers).await {
        Ok((status, headers, body)) => {
            if status.is_success() {
                let cached_response = CachedResponse::new(status, headers, body);
                state
                    .isr_cache
                    .insert(cache_key.clone(), cached_response)
                    .await;
                tracing::debug!(path = %cache_key, "Cache entry refreshed");
            } else {
                tracing::warn!(
                    path = %cache_key,
                    status = status.as_u16(),
                    "Background refresh returned non-success status, keeping stale entry"
                );
            }
        }
        Err(err) => {
            tracing::warn!(
                path = %cache_key,
                error = %err,
                "Background refresh failed, keeping stale entry"
            );
        }
    }

    // Mark refresh as complete
    state.isr_cache.end_refresh(&cache_key);
}

/// Log ISR request with appropriate level based on status
fn log_isr_request(path: &str, status: StatusCode, duration_ms: u64, cache: &str) {
    let is_static = utils::is_static_asset(path);
    let is_page = utils::is_page_route(path);

    match (status.as_u16(), is_static, is_page) {
        (200..=299, true, _) => {
            tracing::trace!(status = status.as_u16(), duration_ms, cache, "ISR request");
        }
        (404, true, _) => {
            tracing::warn!(
                status = status.as_u16(),
                duration_ms,
                cache,
                "ISR request - missing asset"
            );
        }
        (500..=599, true, _) => {
            tracing::error!(
                status = status.as_u16(),
                duration_ms,
                cache,
                "ISR request - server error"
            );
        }
        (200..=299, _, true) => {
            tracing::debug!(status = status.as_u16(), duration_ms, cache, "ISR request");
        }
        (404, _, true) => {}
        (500..=599, _, _) => {
            tracing::error!(
                status = status.as_u16(),
                duration_ms,
                cache,
                "ISR request - server error"
            );
        }
        _ => {
            tracing::debug!(status = status.as_u16(), duration_ms, cache, "ISR request");
        }
    }
}

/// Proxy a request to Bun SSR
pub async fn proxy_to_bun(
    path: &str,
    state: Arc<AppState>,
    forward_headers: HeaderMap,
) -> Result<(StatusCode, HeaderMap, axum::body::Bytes), ProxyError> {
    // Build request with forwarded headers
    let mut request_builder = state.client.get(path);
    for (name, value) in forward_headers.iter() {
        request_builder = request_builder.header(name, value);
    }

    let response = request_builder.send().await.map_err(ProxyError::Network)?;

    let status = StatusCode::from_u16(response.status().as_u16())
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    let mut headers = HeaderMap::new();
    for (name, value) in response.headers() {
        let name_str = name.as_str();
        if name_str == "transfer-encoding"
            || name_str == "connection"
            || name_str == "content-length"
        {
            continue;
        }

        if let Ok(header_name) = axum::http::HeaderName::try_from(name.as_str())
            && let Ok(header_value) = axum::http::HeaderValue::try_from(value.as_bytes())
        {
            headers.insert(header_name, header_value);
        }
    }

    let body = response.bytes().await.map_err(ProxyError::Network)?;
    Ok((status, headers, body))
}

/// Perform health check on Bun SSR and database
pub async fn perform_health_check(
    client: crate::http::HttpClient,
    pool: Option<sqlx::PgPool>,
) -> bool {
    let bun_healthy = match tokio::time::timeout(
        Duration::from_secs(5),
        client.get("/internal/health").send(),
    )
    .await
    {
        Ok(Ok(response)) => {
            let is_success = response.status().is_success();
            if !is_success {
                tracing::warn!(
                    status = response.status().as_u16(),
                    "Health check failed: Bun returned non-success status"
                );
            }
            is_success
        }
        Ok(Err(err)) => {
            tracing::error!(error = %err, "Health check failed: cannot reach Bun");
            false
        }
        Err(_) => {
            tracing::error!(timeout_sec = 5, "Health check timed out");
            false
        }
    };

    // Check database
    let db_healthy = if let Some(pool) = pool {
        match db::health_check(&pool).await {
            Ok(_) => true,
            Err(err) => {
                tracing::error!(error = %err, "Database health check failed");
                false
            }
        }
    } else {
        true
    };

    bun_healthy && db_healthy
}

/// Check if path should trigger tarpit
fn should_tarpit(state: &TarpitState, path: &str) -> bool {
    state.config.enabled && tarpit::is_malicious_path(path)
}

/// Common handler logic for requests with optional peer info
async fn handle_request_with_optional_peer(
    state: Arc<AppState>,
    peer: Option<SocketAddr>,
    req: Request,
) -> Response {
    let path = req.uri().path();

    if should_tarpit(&state.tarpit_state, path) {
        let peer_info = peer.map(ConnectInfo);
        tarpit::tarpit_handler(State(state.tarpit_state.clone()), peer_info, req).await
    } else {
        isr_handler(State(state), req).await
    }
}

/// Fallback handler for TCP connections (has access to peer IP)
pub async fn fallback_handler_tcp(
    State(state): State<Arc<AppState>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    req: Request,
) -> Response {
    handle_request_with_optional_peer(state, Some(peer), req).await
}

/// Fallback handler for Unix sockets (no peer IP available)
pub async fn fallback_handler_unix(State(state): State<Arc<AppState>>, req: Request) -> Response {
    handle_request_with_optional_peer(state, None, req).await
}
