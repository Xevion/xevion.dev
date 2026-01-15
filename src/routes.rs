use axum::{
    Router,
    body::Body,
    extract::Request,
    http::{Method, Uri},
    response::IntoResponse,
    routing::{any, delete, get, post, put},
};
use std::sync::Arc;

use crate::{assets, handlers, state::AppState};

/// Build API routes
pub fn api_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", any(api_root_404_handler))
        .route(
            "/health",
            get(handlers::health_handler).head(handlers::health_handler),
        )
        // Authentication endpoints (public)
        .route("/login", post(handlers::api_login_handler))
        .route("/logout", post(handlers::api_logout_handler))
        .route("/session", get(handlers::api_session_handler))
        // Projects - GET is public (shows all for admin, only non-hidden for public)
        // POST/PUT/DELETE require authentication
        .route(
            "/projects",
            get(handlers::projects_handler).post(handlers::create_project_handler),
        )
        .route(
            "/projects/{id}",
            get(handlers::get_project_handler)
                .put(handlers::update_project_handler)
                .delete(handlers::delete_project_handler),
        )
        // Project tags - authentication checked in handlers
        .route(
            "/projects/{id}/tags",
            get(handlers::get_project_tags_handler).post(handlers::add_project_tag_handler),
        )
        .route(
            "/projects/{id}/tags/{tag_id}",
            delete(handlers::remove_project_tag_handler),
        )
        // Project media - GET is public, POST/PUT/DELETE require authentication
        .route(
            "/projects/{id}/media",
            get(handlers::get_project_media_handler).post(handlers::upload_media_handler),
        )
        .route(
            "/projects/{id}/media/reorder",
            put(handlers::reorder_media_handler),
        )
        .route(
            "/projects/{id}/media/{media_id}",
            delete(handlers::delete_media_handler),
        )
        // Tags - authentication checked in handlers
        .route(
            "/tags",
            get(handlers::list_tags_handler).post(handlers::create_tag_handler),
        )
        .route(
            "/tags/{slug}",
            get(handlers::get_tag_handler).put(handlers::update_tag_handler),
        )
        .route(
            "/tags/{slug}/related",
            get(handlers::get_related_tags_handler),
        )
        .route(
            "/tags/recalculate-cooccurrence",
            post(handlers::recalculate_cooccurrence_handler),
        )
        // Admin stats - requires authentication
        .route("/stats", get(handlers::get_admin_stats_handler))
        // Site settings - GET is public, PUT requires authentication
        .route(
            "/settings",
            get(handlers::get_settings_handler).put(handlers::update_settings_handler),
        )
        // Icon API - proxy to SvelteKit (authentication handled by SvelteKit)
        .route("/icons/{*path}", get(handlers::proxy_icons_handler))
        .fallback(api_404_and_method_handler)
}

/// Build base router (shared routes for all listen addresses)
pub fn build_base_router() -> Router<Arc<AppState>> {
    Router::new()
        .nest("/api", api_routes())
        .route("/api/", any(api_root_404_handler))
        // Serve env.js explicitly before the wildcard (it's at build root, not in client/)
        .route("/_app/env.js", get(handlers::serve_env_js))
        .route(
            "/_app/{*path}",
            get(assets::serve_embedded_asset).head(assets::serve_embedded_asset),
        )
        .route("/pgp", get(handlers::handle_pgp_route))
        .route("/publickey.asc", get(handlers::serve_pgp_key))
        .route("/pgp.asc", get(handlers::serve_pgp_key))
        .route("/.well-known/pgpkey.asc", get(handlers::serve_pgp_key))
        .route("/keys", get(handlers::redirect_to_pgp))
}

async fn api_root_404_handler(uri: Uri) -> impl IntoResponse {
    api_404_handler(uri).await
}

async fn api_404_and_method_handler(req: Request) -> impl IntoResponse {
    use axum::{Json, http::StatusCode};

    let method = req.method();
    let uri = req.uri();
    let path = uri.path();

    if method != Method::GET && method != Method::HEAD && method != Method::OPTIONS {
        let content_type = req
            .headers()
            .get(axum::http::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok());

        if let Some(ct) = content_type {
            if !ct.starts_with("application/json") {
                return (
                    StatusCode::UNSUPPORTED_MEDIA_TYPE,
                    Json(serde_json::json!({
                        "error": "Unsupported media type",
                        "message": "API endpoints only accept application/json"
                    })),
                )
                    .into_response();
            }
        } else if method == Method::POST || method == Method::PUT || method == Method::PATCH {
            // POST/PUT/PATCH require Content-Type header
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Missing Content-Type header",
                    "message": "Content-Type: application/json is required"
                })),
            )
                .into_response();
        }
    }

    // Route not found
    tracing::warn!(path = %path, method = %method, "API route not found");
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "error": "Not found",
            "path": path
        })),
    )
        .into_response()
}

async fn api_404_handler(uri: Uri) -> impl IntoResponse {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();

    api_404_and_method_handler(req).await
}
