use axum::{
    Json,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use std::sync::Arc;

use crate::{assets, proxy, state::AppState, utils};

/// Serve SvelteKit's env.js for dynamic public environment variables.
/// Required for prerendered pages that use `$env/dynamic/public` imports.
pub async fn serve_env_js() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("application/javascript; charset=utf-8"),
    );
    headers.insert(
        axum::http::header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("public, max-age=3600"),
    );

    (StatusCode::OK, headers, assets::get_env_js())
}

/// Serve PGP public key
pub async fn serve_pgp_key() -> impl IntoResponse {
    if let Some(content) = assets::get_static_file("publickey.asc") {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_static("application/pgp-keys"),
        );
        headers.insert(
            axum::http::header::CONTENT_DISPOSITION,
            axum::http::HeaderValue::from_static("attachment; filename=\"publickey.asc\""),
        );
        headers.insert(
            axum::http::header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static("public, max-age=86400"),
        );
        (StatusCode::OK, headers, content).into_response()
    } else {
        (StatusCode::NOT_FOUND, "PGP key not found").into_response()
    }
}

/// Redirect /keys to /pgp
pub async fn redirect_to_pgp() -> impl IntoResponse {
    axum::response::Redirect::permanent("/pgp")
}

/// Handle /pgp route - serve HTML page or raw key based on User-Agent
pub async fn handle_pgp_route(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Request,
) -> Response {
    if utils::prefers_raw_content(&headers) {
        // Serve raw .asc file for CLI tools
        serve_pgp_key().await.into_response()
    } else {
        // Proxy to Bun for HTML page
        proxy::isr_handler(State(state), req).await
    }
}

/// Proxy icon requests to SvelteKit
pub async fn proxy_icons_handler(
    State(state): State<Arc<AppState>>,
    jar: axum_extra::extract::CookieJar,
    axum::extract::Path(path): axum::extract::Path<String>,
    req: Request,
) -> impl IntoResponse {
    let full_path = format!("/api/icons/{}", path);
    let query = req.uri().query().unwrap_or("");

    let path_with_query = if query.is_empty() {
        full_path.clone()
    } else {
        format!("{full_path}?{query}")
    };

    // Build trusted headers with session info
    let mut forward_headers = HeaderMap::new();

    if let Some(cookie) = jar.get("admin_session") {
        if let Ok(session_id) = ulid::Ulid::from_string(cookie.value()) {
            if let Some(session) = state.session_manager.validate_session(session_id) {
                if let Ok(username_value) = axum::http::HeaderValue::from_str(&session.username) {
                    forward_headers.insert("x-session-user", username_value);
                }
            }
        }
    }

    match proxy::proxy_to_bun(&path_with_query, state, forward_headers).await {
        Ok((status, headers, body)) => (status, headers, body).into_response(),
        Err(err) => {
            tracing::error!(error = %err, path = %full_path, "Failed to proxy icon request");
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({
                    "error": "Failed to fetch icon data"
                })),
            )
                .into_response()
        }
    }
}
