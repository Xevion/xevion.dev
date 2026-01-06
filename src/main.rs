use axum::{
    Json, Router,
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::any,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer, trace::TraceLayer};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod assets;
mod config;
mod formatter;
mod health;
mod middleware;
mod og;
mod r2;
mod tarpit;
use assets::serve_embedded_asset;
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
    init_tracing();

    let args = Args::parse();

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

    let health_checker = Arc::new(HealthChecker::new(move || {
        let downstream_url = downstream_url_for_health.clone();
        let http_client = http_client_for_health.clone();
        let unix_client = unix_client_for_health.clone();

        async move { perform_health_check(downstream_url, http_client, unix_client).await }
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
        .route(
            "/projects",
            axum::routing::get(projects_handler).head(projects_handler),
        )
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectLink {
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Project {
    id: String,
    name: String,
    #[serde(rename = "shortDescription")]
    short_description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
    links: Vec<ProjectLink>,
}

async fn projects_handler() -> impl IntoResponse {
    let projects = vec![
        Project {
            id: "1".to_string(),
            name: "xevion.dev".to_string(),
            short_description: "Personal portfolio with fuzzy tag discovery".to_string(),
            icon: None,
            links: vec![ProjectLink {
                url: "https://github.com/Xevion/xevion.dev".to_string(),
                title: Some("GitHub".to_string()),
            }],
        },
        Project {
            id: "2".to_string(),
            name: "Contest".to_string(),
            short_description: "Competitive programming problem archive".to_string(),
            icon: None,
            links: vec![
                ProjectLink {
                    url: "https://github.com/Xevion/contest".to_string(),
                    title: Some("GitHub".to_string()),
                },
                ProjectLink {
                    url: "https://contest.xevion.dev".to_string(),
                    title: Some("Demo".to_string()),
                },
            ],
        },
    ];

    Json(projects)
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

    let start = std::time::Instant::now();

    match proxy_to_bun(&bun_url, state.clone()).await {
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
) -> Result<(StatusCode, HeaderMap, axum::body::Bytes), ProxyError> {
    let client = if state.unix_client.is_some() {
        state.unix_client.as_ref().unwrap()
    } else {
        &state.http_client
    };

    let response = client.get(url).send().await.map_err(ProxyError::Network)?;

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
    }
}
