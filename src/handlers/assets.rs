use axum::{
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
