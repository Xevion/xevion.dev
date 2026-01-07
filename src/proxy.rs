use axum::{
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use std::{net::SocketAddr, sync::Arc, time::Duration};

use crate::{
    assets, db,
    state::{AppState, ProxyError},
    tarpit::{self, TarpitState},
    utils,
};

/// ISR handler - serves pages through Bun SSR with session validation
#[tracing::instrument(skip(state, req), fields(path = %req.uri().path(), method = %req.method()))]
pub async fn isr_handler(State(state): State<Arc<AppState>>, req: Request) -> Response {
    let method = req.method().clone();
    let uri = req.uri();
    let path = uri.path();
    let query = uri.query().unwrap_or("");

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
        tracing::error!("API request reached ISR handler - routing bug!");
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
    if utils::is_static_asset(path) {
        if let Some(response) = assets::try_serve_embedded_asset(path) {
            return response;
        }
        // If not found in embedded assets, continue to proxy (might be in Bun's static dir)
    }

    // Check if this is a prerendered page
    if let Some(response) = assets::try_serve_prerendered_page(path) {
        tracing::debug!(path = %path, "Serving prerendered page");
        return response;
    }

    let path_with_query = if query.is_empty() {
        path.to_string()
    } else {
        format!("{path}?{query}")
    };

    // Build trusted headers to forward to downstream
    let mut forward_headers = HeaderMap::new();

    // SECURITY: Strip any X-Session-User header from incoming request to prevent spoofing

    // Extract and validate session from cookie
    if let Some(cookie_header) = req.headers().get(axum::http::header::COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            // Parse cookies manually to find admin_session
            for cookie_pair in cookie_str.split(';') {
                let cookie_pair = cookie_pair.trim();
                if let Some((name, value)) = cookie_pair.split_once('=') {
                    if name == "admin_session" {
                        // Found session cookie, validate it
                        if let Ok(session_id) = ulid::Ulid::from_string(value) {
                            if let Some(session) =
                                state.session_manager.validate_session(session_id)
                            {
                                // Session is valid - add trusted header
                                if let Ok(username_value) =
                                    axum::http::HeaderValue::from_str(&session.username)
                                {
                                    forward_headers.insert("x-session-user", username_value);
                                }
                            }
                        }
                        break;
                    }
                }
            }
        }
    }

    let start = std::time::Instant::now();

    match proxy_to_bun(&path_with_query, state.clone(), forward_headers).await {
        Ok((status, headers, body)) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let cache = "miss";

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
            tracing::error!("Health check failed: timeout after 5s");
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
