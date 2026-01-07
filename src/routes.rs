use axum::{Router, extract::Request, http::Uri, response::IntoResponse, routing::any};
use std::sync::Arc;

use crate::{assets, handlers, state::AppState};

/// Build API routes
pub fn api_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", any(api_root_404_handler))
        .route(
            "/health",
            axum::routing::get(handlers::health_handler).head(handlers::health_handler),
        )
        // Authentication endpoints (public)
        .route("/login", axum::routing::post(handlers::api_login_handler))
        .route("/logout", axum::routing::post(handlers::api_logout_handler))
        .route(
            "/session",
            axum::routing::get(handlers::api_session_handler),
        )
        // Projects - GET is public (shows all for admin, only non-hidden for public)
        // POST/PUT/DELETE require authentication
        .route(
            "/projects",
            axum::routing::get(handlers::projects_handler).post(handlers::create_project_handler),
        )
        .route(
            "/projects/{id}",
            axum::routing::get(handlers::get_project_handler)
                .put(handlers::update_project_handler)
                .delete(handlers::delete_project_handler),
        )
        // Project tags - authentication checked in handlers
        .route(
            "/projects/{id}/tags",
            axum::routing::get(handlers::get_project_tags_handler)
                .post(handlers::add_project_tag_handler),
        )
        .route(
            "/projects/{id}/tags/{tag_id}",
            axum::routing::delete(handlers::remove_project_tag_handler),
        )
        // Tags - authentication checked in handlers
        .route(
            "/tags",
            axum::routing::get(handlers::list_tags_handler).post(handlers::create_tag_handler),
        )
        .route(
            "/tags/{slug}",
            axum::routing::get(handlers::get_tag_handler).put(handlers::update_tag_handler),
        )
        .route(
            "/tags/{slug}/related",
            axum::routing::get(handlers::get_related_tags_handler),
        )
        .route(
            "/tags/recalculate-cooccurrence",
            axum::routing::post(handlers::recalculate_cooccurrence_handler),
        )
        // Admin stats - requires authentication
        .route(
            "/stats",
            axum::routing::get(handlers::get_admin_stats_handler),
        )
        // Site settings - GET is public, PUT requires authentication
        .route(
            "/settings",
            axum::routing::get(handlers::get_settings_handler)
                .put(handlers::update_settings_handler),
        )
        // Icon API - proxy to SvelteKit (authentication handled by SvelteKit)
        .route(
            "/icons/{*path}",
            axum::routing::get(handlers::proxy_icons_handler),
        )
        .fallback(api_404_and_method_handler)
}

/// Build base router (shared routes for all listen addresses)
pub fn build_base_router() -> Router<Arc<AppState>> {
    Router::new()
        .nest("/api", api_routes())
        .route("/api/", any(api_root_404_handler))
        .route(
            "/_app/{*path}",
            axum::routing::get(assets::serve_embedded_asset).head(assets::serve_embedded_asset),
        )
        .route("/pgp", axum::routing::get(handlers::handle_pgp_route))
        .route(
            "/publickey.asc",
            axum::routing::get(handlers::serve_pgp_key),
        )
        .route("/pgp.asc", axum::routing::get(handlers::serve_pgp_key))
        .route(
            "/.well-known/pgpkey.asc",
            axum::routing::get(handlers::serve_pgp_key),
        )
        .route("/keys", axum::routing::get(handlers::redirect_to_pgp))
}

async fn api_root_404_handler(uri: Uri) -> impl IntoResponse {
    api_404_handler(uri).await
}

async fn api_404_and_method_handler(req: Request) -> impl IntoResponse {
    use axum::{Json, http::StatusCode};

    let method = req.method();
    let uri = req.uri();
    let path = uri.path();

    if method != axum::http::Method::GET
        && method != axum::http::Method::HEAD
        && method != axum::http::Method::OPTIONS
    {
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
        } else if method == axum::http::Method::POST
            || method == axum::http::Method::PUT
            || method == axum::http::Method::PATCH
        {
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
    let req = Request::builder()
        .uri(uri)
        .body(axum::body::Body::empty())
        .unwrap();

    api_404_and_method_handler(req).await
}
