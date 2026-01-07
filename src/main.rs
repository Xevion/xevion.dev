use axum::{
    Json, Router,
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::any,
};
use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer, trace::TraceLayer};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod assets;
mod auth;
mod config;
mod db;
mod formatter;
mod health;
mod middleware;
mod og;
mod r2;
mod tarpit;
use assets::{serve_embedded_asset, try_serve_embedded_asset};
use config::{Args, ListenAddr};
use formatter::{CustomJsonFormatter, CustomPrettyFormatter};
use health::HealthChecker;
use middleware::RequestIdLayer;
use tarpit::{TarpitConfig, TarpitState, is_malicious_path, tarpit_handler};

fn init_tracing() {
    let use_json = std::env::var("LOG_JSON")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    let filter = if let Ok(rust_log) = std::env::var("RUST_LOG") {
        EnvFilter::new(rust_log)
    } else {
        let our_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| {
            if cfg!(debug_assertions) {
                "debug".to_string()
            } else {
                "info".to_string()
            }
        });

        EnvFilter::new(format!("warn,api={our_level}"))
    };

    if use_json {
        tracing_subscriber::registry()
            .with(filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .event_format(CustomJsonFormatter)
                    .fmt_fields(tracing_subscriber::fmt::format::DefaultFields::new())
                    .with_ansi(false),
            )
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer().event_format(CustomPrettyFormatter))
            .init();
    }
}

#[tokio::main]
async fn main() {
    // Load .env file if present
    dotenvy::dotenv().ok();

    // Parse args early to allow --help to work without database
    let args = Args::parse();

    init_tracing();

    // Load database URL from environment (fail-fast)
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in environment");

    // Create connection pool
    let pool = db::create_pool(&database_url)
        .await
        .expect("Failed to connect to database");

    // Run migrations on startup
    tracing::info!("Running database migrations...");
    sqlx::migrate!().run(&pool).await.unwrap_or_else(|e| {
        tracing::error!(error = %e, "Migration failed");
        std::process::exit(1);
    });

    tracing::info!("Migrations applied successfully");

    // Ensure admin user exists
    auth::ensure_admin_user(&pool)
        .await
        .expect("Failed to ensure admin user exists");

    // Initialize session manager
    let session_manager = Arc::new(
        auth::SessionManager::new(pool.clone())
            .await
            .expect("Failed to initialize session manager"),
    );

    // Spawn background task to cleanup expired sessions
    tokio::spawn({
        let session_manager = session_manager.clone();
        async move {
            let mut interval = tokio::time::interval(Duration::from_secs(3600)); // Every hour
            loop {
                interval.tick().await;
                if let Err(e) = session_manager.cleanup_expired().await {
                    tracing::error!(error = %e, "Failed to cleanup expired sessions");
                }
            }
        }
    });

    if args.listen.is_empty() {
        eprintln!("Error: At least one --listen address is required");
        std::process::exit(1);
    }

    // Create HTTP client for TCP connections with optimized pool settings
    let http_client = reqwest::Client::builder()
        .pool_max_idle_per_host(8)
        .pool_idle_timeout(Duration::from_secs(600)) // 10 minutes
        .tcp_keepalive(Some(Duration::from_secs(60)))
        .timeout(Duration::from_secs(5)) // Default timeout for SSR
        .connect_timeout(Duration::from_secs(3))
        .redirect(reqwest::redirect::Policy::none()) // Don't follow redirects - pass them through
        .build()
        .expect("Failed to create HTTP client");

    // Create Unix socket client if downstream is a Unix socket
    let unix_client = if args.downstream.starts_with('/') || args.downstream.starts_with("./") {
        let path = PathBuf::from(&args.downstream);
        Some(
            reqwest::Client::builder()
                .pool_max_idle_per_host(8)
                .pool_idle_timeout(Duration::from_secs(600)) // 10 minutes
                .timeout(Duration::from_secs(5)) // Default timeout for SSR
                .connect_timeout(Duration::from_secs(3))
                .redirect(reqwest::redirect::Policy::none()) // Don't follow redirects - pass them through
                .unix_socket(path)
                .build()
                .expect("Failed to create Unix socket client"),
        )
    } else {
        None
    };

    // Create health checker
    let downstream_url_for_health = args.downstream.clone();
    let http_client_for_health = http_client.clone();
    let unix_client_for_health = unix_client.clone();
    let pool_for_health = pool.clone();

    let health_checker = Arc::new(HealthChecker::new(move || {
        let downstream_url = downstream_url_for_health.clone();
        let http_client = http_client_for_health.clone();
        let unix_client = unix_client_for_health.clone();
        let pool = pool_for_health.clone();

        async move { perform_health_check(downstream_url, http_client, unix_client, Some(pool)).await }
    }));

    let tarpit_config = TarpitConfig::from_env();
    let tarpit_state = Arc::new(TarpitState::new(tarpit_config));

    tracing::info!(
        enabled = tarpit_state.config.enabled,
        delay_range_ms = format!(
            "{}-{}",
            tarpit_state.config.delay_min_ms, tarpit_state.config.delay_max_ms
        ),
        max_global = tarpit_state.config.max_global_connections,
        max_per_ip = tarpit_state.config.max_connections_per_ip,
        "Tarpit initialized"
    );

    let state = Arc::new(AppState {
        downstream_url: args.downstream.clone(),
        http_client,
        unix_client,
        health_checker,
        tarpit_state,
        pool: pool.clone(),
        session_manager: session_manager.clone(),
    });

    // Regenerate common OGP images on startup
    tokio::spawn({
        let state = state.clone();
        async move {
            og::regenerate_common_images(state).await;
        }
    });

    // Build base router (shared routes)
    fn build_base_router() -> Router<Arc<AppState>> {
        Router::new()
            .nest("/api", api_routes())
            .route("/api/", any(api_root_404_handler))
            .route(
                "/_app/{*path}",
                axum::routing::get(serve_embedded_asset).head(serve_embedded_asset),
            )
    }

    fn apply_middleware(
        router: Router<Arc<AppState>>,
        trust_request_id: Option<String>,
    ) -> Router<Arc<AppState>> {
        router
            .layer(TraceLayer::new_for_http())
            .layer(RequestIdLayer::new(trust_request_id))
            .layer(CorsLayer::permissive())
            .layer(RequestBodyLimitLayer::new(1_048_576))
    }

    let mut tasks = Vec::new();

    for listen_addr in &args.listen {
        let state = state.clone();
        let trust_request_id = args.trust_request_id.clone();
        let listen_addr = listen_addr.clone();

        let task = tokio::spawn(async move {
            match listen_addr {
                ListenAddr::Tcp(addr) => {
                    let app = apply_middleware(
                        build_base_router().fallback(fallback_handler_tcp),
                        trust_request_id,
                    )
                    .with_state(state);

                    let listener = tokio::net::TcpListener::bind(addr)
                        .await
                        .expect("Failed to bind TCP listener");

                    let url = if addr.is_ipv6() {
                        format!("http://[{}]:{}", addr.ip(), addr.port())
                    } else {
                        format!("http://{}:{}", addr.ip(), addr.port())
                    };

                    tracing::info!(url, "Listening on TCP");
                    axum::serve(
                        listener,
                        app.into_make_service_with_connect_info::<SocketAddr>(),
                    )
                    .await
                    .expect("Server error on TCP listener");
                }
                ListenAddr::Unix(path) => {
                    let app = apply_middleware(
                        build_base_router().fallback(fallback_handler_unix),
                        trust_request_id,
                    )
                    .with_state(state);

                    let _ = std::fs::remove_file(&path);

                    let listener = tokio::net::UnixListener::bind(&path)
                        .expect("Failed to bind Unix socket listener");

                    tracing::info!(socket = %path.display(), "Listening on Unix socket");
                    axum::serve(listener, app)
                        .await
                        .expect("Server error on Unix socket listener");
                }
            }
        });

        tasks.push(task);
    }

    for task in tasks {
        task.await.expect("Listener task panicked");
    }
}

#[derive(Clone)]
pub struct AppState {
    downstream_url: String,
    http_client: reqwest::Client,
    unix_client: Option<reqwest::Client>,
    health_checker: Arc<HealthChecker>,
    tarpit_state: Arc<TarpitState>,
    pool: sqlx::PgPool,
    session_manager: Arc<auth::SessionManager>,
}

#[derive(Debug)]
pub enum ProxyError {
    Network(reqwest::Error),
    Other(String),
}

impl std::fmt::Display for ProxyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyError::Network(e) => write!(f, "Network error: {e}"),
            ProxyError::Other(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for ProxyError {}

fn is_static_asset(path: &str) -> bool {
    path.starts_with("/node_modules/")
        || path.starts_with("/@") // Vite internals like /@vite/client, /@fs/, /@id/
        || path.starts_with("/.svelte-kit/")
        || path.starts_with("/.well-known/")
        || path.ends_with(".woff2")
        || path.ends_with(".woff")
        || path.ends_with(".ttf")
        || path.ends_with(".ico")
        || path.ends_with(".png")
        || path.ends_with(".jpg")
        || path.ends_with(".svg")
        || path.ends_with(".webp")
        || path.ends_with(".css")
        || path.ends_with(".js")
        || path.ends_with(".map")
}

fn is_page_route(path: &str) -> bool {
    !path.starts_with("/node_modules/")
        && !path.starts_with("/@")
        && !path.starts_with("/.svelte-kit/")
        && !path.contains('.')
}

fn api_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", any(api_root_404_handler))
        .route(
            "/health",
            axum::routing::get(health_handler).head(health_handler),
        )
        // Authentication endpoints (public)
        .route("/login", axum::routing::post(api_login_handler))
        .route("/logout", axum::routing::post(api_logout_handler))
        .route("/session", axum::routing::get(api_session_handler))
        // Projects - GET is public, other methods require auth
        .route("/projects", axum::routing::get(projects_handler))
        // Project tags - authentication checked in handlers
        .route(
            "/projects/{id}/tags",
            axum::routing::get(get_project_tags_handler).post(add_project_tag_handler),
        )
        .route(
            "/projects/{id}/tags/{tag_id}",
            axum::routing::delete(remove_project_tag_handler),
        )
        // Tags - authentication checked in handlers
        .route(
            "/tags",
            axum::routing::get(list_tags_handler).post(create_tag_handler),
        )
        .route(
            "/tags/{slug}",
            axum::routing::get(get_tag_handler).put(update_tag_handler),
        )
        .route(
            "/tags/{slug}/related",
            axum::routing::get(get_related_tags_handler),
        )
        .route(
            "/tags/recalculate-cooccurrence",
            axum::routing::post(recalculate_cooccurrence_handler),
        )
        // Icon API - proxy to SvelteKit (authentication handled by SvelteKit)
        .route("/icons/{*path}", axum::routing::get(proxy_icons_handler))
        .fallback(api_404_and_method_handler)
}

async fn api_root_404_handler(uri: axum::http::Uri) -> impl IntoResponse {
    api_404_handler(uri).await
}

fn accepts_html(headers: &HeaderMap) -> bool {
    if let Some(accept) = headers.get(axum::http::header::ACCEPT) {
        if let Ok(accept_str) = accept.to_str() {
            return accept_str.contains("text/html") || accept_str.contains("*/*");
        }
    }
    // Default to true for requests without Accept header (browsers typically send it)
    true
}

fn serve_error_page(status: StatusCode) -> Response {
    let status_code = status.as_u16();

    if let Some(html) = assets::get_error_page(status_code) {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_static("text/html; charset=utf-8"),
        );
        headers.insert(
            axum::http::header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static("no-cache, no-store, must-revalidate"),
        );

        (status, headers, html).into_response()
    } else {
        // Fallback for undefined error codes (500 generic page)
        tracing::warn!(
            status_code,
            "No prerendered error page found for status code - using fallback"
        );

        if let Some(fallback_html) = assets::get_error_page(500) {
            let mut headers = HeaderMap::new();
            headers.insert(
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderValue::from_static("text/html; charset=utf-8"),
            );
            headers.insert(
                axum::http::header::CACHE_CONTROL,
                axum::http::HeaderValue::from_static("no-cache, no-store, must-revalidate"),
            );

            (status, headers, fallback_html).into_response()
        } else {
            // Last resort: plaintext (should never happen if 500.html exists)
            (status, format!("Error {}", status_code)).into_response()
        }
    }
}

async fn health_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let healthy = state.health_checker.check().await;

    if healthy {
        (StatusCode::OK, "OK")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "Unhealthy")
    }
}

async fn api_404_and_method_handler(req: Request) -> impl IntoResponse {
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

async fn api_404_handler(uri: axum::http::Uri) -> impl IntoResponse {
    let req = Request::builder()
        .uri(uri)
        .body(axum::body::Body::empty())
        .unwrap();

    api_404_and_method_handler(req).await
}

async fn projects_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match db::get_public_projects(&state.pool).await {
        Ok(projects) => {
            let api_projects: Vec<db::ApiProject> =
                projects.into_iter().map(|p| p.to_api_project()).collect();
            Json(api_projects).into_response()
        }
        Err(err) => {
            tracing::error!(error = %err, "Failed to fetch projects from database");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to fetch projects"
                })),
            )
                .into_response()
        }
    }
}

// Icon API handler - proxy to SvelteKit
async fn proxy_icons_handler(
    State(state): State<Arc<AppState>>,
    jar: axum_extra::extract::CookieJar,
    axum::extract::Path(path): axum::extract::Path<String>,
    req: Request,
) -> impl IntoResponse {
    let full_path = format!("/api/icons/{}", path);
    let query = req.uri().query().unwrap_or("");

    let bun_url = if state.downstream_url.starts_with('/') || state.downstream_url.starts_with("./")
    {
        if query.is_empty() {
            format!("http://localhost{}", full_path)
        } else {
            format!("http://localhost{}?{}", full_path, query)
        }
    } else if query.is_empty() {
        format!("{}{}", state.downstream_url, full_path)
    } else {
        format!("{}{}?{}", state.downstream_url, full_path, query)
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

    match proxy_to_bun(&bun_url, state, forward_headers).await {
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

// Tag API handlers

async fn list_tags_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match db::get_all_tags_with_counts(&state.pool).await {
        Ok(tags_with_counts) => {
            let api_tags: Vec<db::ApiTagWithCount> = tags_with_counts
                .into_iter()
                .map(|(tag, count)| db::ApiTagWithCount {
                    tag: tag.to_api_tag(),
                    project_count: count,
                })
                .collect();
            Json(api_tags).into_response()
        }
        Err(err) => {
            tracing::error!(error = %err, "Failed to fetch tags");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to fetch tags"
                })),
            )
                .into_response()
        }
    }
}

/// Validate hex color format (6 characters, no hash, no alpha)
fn validate_hex_color(color: &str) -> bool {
    color.len() == 6 && color.chars().all(|c| c.is_ascii_hexdigit())
}

#[derive(serde::Deserialize)]
struct CreateTagRequest {
    name: String,
    slug: Option<String>,
    color: Option<String>,
}

async fn create_tag_handler(
    State(state): State<Arc<AppState>>,
    jar: axum_extra::extract::CookieJar,
    Json(payload): Json<CreateTagRequest>,
) -> impl IntoResponse {
    if check_session(&state, &jar).is_none() {
        return require_auth_response().into_response();
    }
    if payload.name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Validation error",
                "message": "Tag name cannot be empty"
            })),
        )
            .into_response();
    }

    // Validate color if provided
    if let Some(ref color) = payload.color {
        if !validate_hex_color(color) {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Validation error",
                    "message": "Invalid color format. Must be 6-character hex (e.g., '3b82f6')"
                })),
            )
                .into_response();
        }
    }

    match db::create_tag(
        &state.pool,
        &payload.name,
        payload.slug.as_deref(),
        payload.color.as_deref(),
    )
    .await
    {
        Ok(tag) => (StatusCode::CREATED, Json(tag.to_api_tag())).into_response(),
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": "Conflict",
                "message": "A tag with this name or slug already exists"
            })),
        )
            .into_response(),
        Err(err) => {
            tracing::error!(error = %err, "Failed to create tag");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to create tag"
                })),
            )
                .into_response()
        }
    }
}

async fn get_tag_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(slug): axum::extract::Path<String>,
) -> impl IntoResponse {
    match db::get_tag_by_slug(&state.pool, &slug).await {
        Ok(Some(tag)) => match db::get_projects_for_tag(&state.pool, tag.id).await {
            Ok(projects) => {
                let response = serde_json::json!({
                    "tag": tag.to_api_tag(),
                    "projects": projects.into_iter().map(|p| p.to_api_project()).collect::<Vec<_>>()
                });
                Json(response).into_response()
            }
            Err(err) => {
                tracing::error!(error = %err, "Failed to fetch projects for tag");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": "Internal server error",
                        "message": "Failed to fetch projects"
                    })),
                )
                    .into_response()
            }
        },
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Not found",
                "message": "Tag not found"
            })),
        )
            .into_response(),
        Err(err) => {
            tracing::error!(error = %err, "Failed to fetch tag");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to fetch tag"
                })),
            )
                .into_response()
        }
    }
}

#[derive(serde::Deserialize)]
struct UpdateTagRequest {
    name: String,
    slug: Option<String>,
    color: Option<String>,
}

async fn update_tag_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(slug): axum::extract::Path<String>,
    jar: axum_extra::extract::CookieJar,
    Json(payload): Json<UpdateTagRequest>,
) -> impl IntoResponse {
    if check_session(&state, &jar).is_none() {
        return require_auth_response().into_response();
    }
    if payload.name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Validation error",
                "message": "Tag name cannot be empty"
            })),
        )
            .into_response();
    }

    // Validate color if provided
    if let Some(ref color) = payload.color {
        if !validate_hex_color(color) {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Validation error",
                    "message": "Invalid color format. Must be 6-character hex (e.g., '3b82f6')"
                })),
            )
                .into_response();
        }
    }

    let tag = match db::get_tag_by_slug(&state.pool, &slug).await {
        Ok(Some(tag)) => tag,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "Not found",
                    "message": "Tag not found"
                })),
            )
                .into_response();
        }
        Err(err) => {
            tracing::error!(error = %err, "Failed to fetch tag");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to fetch tag"
                })),
            )
                .into_response();
        }
    };

    match db::update_tag(
        &state.pool,
        tag.id,
        &payload.name,
        payload.slug.as_deref(),
        payload.color.as_deref(),
    )
    .await
    {
        Ok(updated_tag) => Json(updated_tag.to_api_tag()).into_response(),
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": "Conflict",
                "message": "A tag with this name or slug already exists"
            })),
        )
            .into_response(),
        Err(err) => {
            tracing::error!(error = %err, "Failed to update tag");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to update tag"
                })),
            )
                .into_response()
        }
    }
}

async fn get_related_tags_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(slug): axum::extract::Path<String>,
) -> impl IntoResponse {
    let tag = match db::get_tag_by_slug(&state.pool, &slug).await {
        Ok(Some(tag)) => tag,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "Not found",
                    "message": "Tag not found"
                })),
            )
                .into_response();
        }
        Err(err) => {
            tracing::error!(error = %err, "Failed to fetch tag");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to fetch tag"
                })),
            )
                .into_response();
        }
    };

    match db::get_related_tags(&state.pool, tag.id, 10).await {
        Ok(related_tags) => {
            let api_related_tags: Vec<db::ApiRelatedTag> = related_tags
                .into_iter()
                .map(|(tag, count)| db::ApiRelatedTag {
                    tag: tag.to_api_tag(),
                    cooccurrence_count: count,
                })
                .collect();
            Json(api_related_tags).into_response()
        }
        Err(err) => {
            tracing::error!(error = %err, "Failed to fetch related tags");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to fetch related tags"
                })),
            )
                .into_response()
        }
    }
}

async fn recalculate_cooccurrence_handler(
    State(state): State<Arc<AppState>>,
    jar: axum_extra::extract::CookieJar,
) -> impl IntoResponse {
    if check_session(&state, &jar).is_none() {
        return require_auth_response().into_response();
    }
    match db::recalculate_tag_cooccurrence(&state.pool).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "message": "Tag cooccurrence recalculated successfully"
            })),
        )
            .into_response(),
        Err(err) => {
            tracing::error!(error = %err, "Failed to recalculate cooccurrence");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to recalculate cooccurrence"
                })),
            )
                .into_response()
        }
    }
}

// Authentication API handlers

fn check_session(state: &AppState, jar: &axum_extra::extract::CookieJar) -> Option<auth::Session> {
    let session_cookie = jar.get("admin_session")?;
    let session_id = ulid::Ulid::from_string(session_cookie.value()).ok()?;
    state.session_manager.validate_session(session_id)
}

fn require_auth_response() -> impl IntoResponse {
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({
            "error": "Unauthorized",
            "message": "Authentication required"
        })),
    )
}

#[derive(serde::Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(serde::Serialize)]
struct LoginResponse {
    success: bool,
    username: String,
}

#[derive(serde::Serialize)]
struct SessionResponse {
    authenticated: bool,
    username: String,
}

async fn api_login_handler(
    State(state): State<Arc<AppState>>,
    jar: axum_extra::extract::CookieJar,
    Json(payload): Json<LoginRequest>,
) -> Result<(axum_extra::extract::CookieJar, Json<LoginResponse>), impl IntoResponse> {
    let user = match auth::get_admin_user(&state.pool, &payload.username).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Invalid credentials",
                    "message": "Username or password incorrect"
                })),
            ));
        }
        Err(err) => {
            tracing::error!(error = %err, "Failed to fetch admin user");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to authenticate"
                })),
            ));
        }
    };

    let password_valid = match auth::verify_password(&payload.password, &user.password_hash) {
        Ok(valid) => valid,
        Err(err) => {
            tracing::error!(error = %err, "Failed to verify password");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to authenticate"
                })),
            ));
        }
    };

    if !password_valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "Invalid credentials",
                "message": "Username or password incorrect"
            })),
        ));
    }

    let session = match state
        .session_manager
        .create_session(user.id, user.username.clone())
        .await
    {
        Ok(session) => session,
        Err(err) => {
            tracing::error!(error = %err, "Failed to create session");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to create session"
                })),
            ));
        }
    };

    let cookie =
        axum_extra::extract::cookie::Cookie::build(("admin_session", session.id.to_string()))
            .path("/")
            .http_only(true)
            .same_site(axum_extra::extract::cookie::SameSite::Lax)
            .max_age(time::Duration::days(7))
            .build();

    let jar = jar.add(cookie);

    tracing::info!(username = %user.username, "User logged in");

    Ok((
        jar,
        Json(LoginResponse {
            success: true,
            username: user.username,
        }),
    ))
}

async fn api_logout_handler(
    State(state): State<Arc<AppState>>,
    jar: axum_extra::extract::CookieJar,
) -> (axum_extra::extract::CookieJar, StatusCode) {
    if let Some(cookie) = jar.get("admin_session") {
        if let Ok(session_id) = ulid::Ulid::from_string(cookie.value()) {
            if let Err(e) = state.session_manager.delete_session(session_id).await {
                tracing::error!(error = %e, "Failed to delete session during logout");
            }
        }
    }

    let cookie = axum_extra::extract::cookie::Cookie::build(("admin_session", ""))
        .path("/")
        .max_age(time::Duration::ZERO)
        .build();

    (jar.add(cookie), StatusCode::OK)
}

async fn api_session_handler(
    State(state): State<Arc<AppState>>,
    jar: axum_extra::extract::CookieJar,
) -> impl IntoResponse {
    let session_cookie = jar.get("admin_session");

    let session_id = session_cookie.and_then(|cookie| ulid::Ulid::from_string(cookie.value()).ok());

    let session = session_id.and_then(|id| state.session_manager.validate_session(id));

    match session {
        Some(session) => (
            StatusCode::OK,
            Json(SessionResponse {
                authenticated: true,
                username: session.username,
            }),
        )
            .into_response(),
        None => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "Unauthorized",
                "message": "No valid session"
            })),
        )
            .into_response(),
    }
}

// Project-Tag association handlers

async fn get_project_tags_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let project_id = match uuid::Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid project ID",
                    "message": "Project ID must be a valid UUID"
                })),
            )
                .into_response();
        }
    };

    match db::get_tags_for_project(&state.pool, project_id).await {
        Ok(tags) => {
            let api_tags: Vec<db::ApiTag> = tags.into_iter().map(|t| t.to_api_tag()).collect();
            Json(api_tags).into_response()
        }
        Err(err) => {
            tracing::error!(error = %err, "Failed to fetch tags for project");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to fetch tags"
                })),
            )
                .into_response()
        }
    }
}

#[derive(serde::Deserialize)]
struct AddProjectTagRequest {
    tag_id: String,
}

async fn add_project_tag_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
    jar: axum_extra::extract::CookieJar,
    Json(payload): Json<AddProjectTagRequest>,
) -> impl IntoResponse {
    if check_session(&state, &jar).is_none() {
        return require_auth_response().into_response();
    }
    let project_id = match uuid::Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid project ID",
                    "message": "Project ID must be a valid UUID"
                })),
            )
                .into_response();
        }
    };

    let tag_id = match uuid::Uuid::parse_str(&payload.tag_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid tag ID",
                    "message": "Tag ID must be a valid UUID"
                })),
            )
                .into_response();
        }
    };

    match db::add_tag_to_project(&state.pool, project_id, tag_id).await {
        Ok(()) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "message": "Tag added to project"
            })),
        )
            .into_response(),
        Err(sqlx::Error::Database(db_err)) if db_err.is_foreign_key_violation() => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Not found",
                "message": "Project or tag not found"
            })),
        )
            .into_response(),
        Err(err) => {
            tracing::error!(error = %err, "Failed to add tag to project");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to add tag to project"
                })),
            )
                .into_response()
        }
    }
}

async fn remove_project_tag_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path((id, tag_id)): axum::extract::Path<(String, String)>,
    jar: axum_extra::extract::CookieJar,
) -> impl IntoResponse {
    if check_session(&state, &jar).is_none() {
        return require_auth_response().into_response();
    }
    let project_id = match uuid::Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid project ID",
                    "message": "Project ID must be a valid UUID"
                })),
            )
                .into_response();
        }
    };

    let tag_id = match uuid::Uuid::parse_str(&tag_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid tag ID",
                    "message": "Tag ID must be a valid UUID"
                })),
            )
                .into_response();
        }
    };

    match db::remove_tag_from_project(&state.pool, project_id, tag_id).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "message": "Tag removed from project"
            })),
        )
            .into_response(),
        Err(err) => {
            tracing::error!(error = %err, "Failed to remove tag from project");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to remove tag from project"
                })),
            )
                .into_response()
        }
    }
}

fn should_tarpit(state: &TarpitState, path: &str) -> bool {
    state.config.enabled && is_malicious_path(path)
}

async fn fallback_handler_tcp(
    State(state): State<Arc<AppState>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    req: Request,
) -> Response {
    let path = req.uri().path();

    if should_tarpit(&state.tarpit_state, path) {
        tarpit_handler(
            State(state.tarpit_state.clone()),
            Some(ConnectInfo(peer)),
            req,
        )
        .await
    } else {
        isr_handler(State(state), req).await
    }
}

async fn fallback_handler_unix(State(state): State<Arc<AppState>>, req: Request) -> Response {
    let path = req.uri().path();

    if should_tarpit(&state.tarpit_state, path) {
        tarpit_handler(State(state.tarpit_state.clone()), None, req).await
    } else {
        isr_handler(State(state), req).await
    }
}

#[tracing::instrument(skip(state, req), fields(path = %req.uri().path(), method = %req.method()))]
async fn isr_handler(State(state): State<Arc<AppState>>, req: Request) -> Response {
    let method = req.method().clone();
    let uri = req.uri();
    let path = uri.path();
    let query = uri.query().unwrap_or("");

    if method != axum::http::Method::GET && method != axum::http::Method::HEAD {
        tracing::warn!(method = %method, path = %path, "Non-GET/HEAD request to non-API route");

        if accepts_html(req.headers()) {
            return serve_error_page(StatusCode::METHOD_NOT_ALLOWED);
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

        if accepts_html(req.headers()) {
            return serve_error_page(StatusCode::NOT_FOUND);
        }

        return (StatusCode::NOT_FOUND, "Not found").into_response();
    }

    // Check if this is a static asset that exists in embedded CLIENT_ASSETS
    // This handles root-level files like favicon.ico, favicon.svg, etc.
    if is_static_asset(path) {
        if let Some(response) = try_serve_embedded_asset(path) {
            return response;
        }
        // If not found in embedded assets, continue to proxy (might be in Bun's static dir)
    }

    let bun_url = if state.downstream_url.starts_with('/') || state.downstream_url.starts_with("./")
    {
        if query.is_empty() {
            format!("http://localhost{path}")
        } else {
            format!("http://localhost{path}?{query}")
        }
    } else if query.is_empty() {
        format!("{}{}", state.downstream_url, path)
    } else {
        format!("{}{}?{}", state.downstream_url, path, query)
    };

    // Build trusted headers to forward to downstream
    let mut forward_headers = HeaderMap::new();

    // SECURITY: Strip any X-Session-User header from incoming request to prevent spoofing
    // (We will add it ourselves if session is valid)

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

    match proxy_to_bun(&bun_url, state.clone(), forward_headers).await {
        Ok((status, headers, body)) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let cache = "miss";

            let is_static = is_static_asset(path);
            let is_page = is_page_route(path);

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
            if (status.is_client_error() || status.is_server_error()) && accepts_html(req.headers())
            {
                return serve_error_page(status);
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
                url = %bun_url,
                duration_ms,
                "Failed to proxy to Bun"
            );

            // Serve 502 error page instead of plaintext
            if accepts_html(req.headers()) {
                return serve_error_page(StatusCode::BAD_GATEWAY);
            }

            (
                StatusCode::BAD_GATEWAY,
                format!("Failed to render page: {err}"),
            )
                .into_response()
        }
    }
}

async fn proxy_to_bun(
    url: &str,
    state: Arc<AppState>,
    forward_headers: HeaderMap,
) -> Result<(StatusCode, HeaderMap, axum::body::Bytes), ProxyError> {
    let client = if state.unix_client.is_some() {
        state.unix_client.as_ref().unwrap()
    } else {
        &state.http_client
    };

    // Build request with forwarded headers
    let mut request_builder = client.get(url);
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

async fn perform_health_check(
    downstream_url: String,
    http_client: reqwest::Client,
    unix_client: Option<reqwest::Client>,
    pool: Option<sqlx::PgPool>,
) -> bool {
    let url = if downstream_url.starts_with('/') || downstream_url.starts_with("./") {
        "http://localhost/internal/health".to_string()
    } else {
        format!("{downstream_url}/internal/health")
    };

    let client = if unix_client.is_some() {
        unix_client.as_ref().unwrap()
    } else {
        &http_client
    };

    let bun_healthy =
        match tokio::time::timeout(Duration::from_secs(5), client.get(&url).send()).await {
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
