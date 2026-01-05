use axum::{
    Json, Router,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod formatter;
mod middleware;
use config::{Args, ListenAddr};
use formatter::{CustomJsonFormatter, CustomPrettyFormatter};
use middleware::RequestIdLayer;

fn init_tracing() {
    let use_json = std::env::var("LOG_JSON")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    // Build the EnvFilter
    // Priority: RUST_LOG > LOG_LEVEL > default
    let filter = if let Ok(rust_log) = std::env::var("RUST_LOG") {
        // RUST_LOG overwrites everything
        EnvFilter::new(rust_log)
    } else {
        // Get LOG_LEVEL for our crate, default based on build profile
        let our_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| {
            if cfg!(debug_assertions) {
                "debug".to_string()
            } else {
                "info".to_string()
            }
        });

        // Default other crates to WARN, our crate to LOG_LEVEL
        EnvFilter::new(format!("warn,api={}", our_level))
    };

    if use_json {
        tracing_subscriber::registry()
            .with(filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .event_format(CustomJsonFormatter)
                    .fmt_fields(tracing_subscriber::fmt::format::DefaultFields::new())
                    .with_ansi(false), // Disable ANSI codes in JSON mode
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
    // Initialize tracing with configurable format and levels
    init_tracing();

    // Parse CLI arguments and environment variables
    let args = Args::parse();

    // Validate we have at least one listen address
    if args.listen.is_empty() {
        eprintln!("Error: At least one --listen address is required");
        std::process::exit(1);
    }

    // Create shared application state
    let state = Arc::new(AppState {
        downstream_url: args.downstream.clone(),
    });

    // Build router with shared state
    let app = Router::new()
        .nest("/api", api_routes().fallback(api_404_handler))
        .fallback(isr_handler)
        .layer(TraceLayer::new_for_http())
        .layer(RequestIdLayer::new(args.trust_request_id.clone()))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Spawn a listener for each address
    let mut tasks = Vec::new();

    for listen_addr in &args.listen {
        let app = app.clone();
        let listen_addr = listen_addr.clone();

        let task = tokio::spawn(async move {
            match listen_addr {
                ListenAddr::Tcp(addr) => {
                    let listener = tokio::net::TcpListener::bind(addr)
                        .await
                        .expect("Failed to bind TCP listener");

                    // Format as clickable URL
                    let url = if addr.is_ipv6() {
                        format!("http://[{}]:{}", addr.ip(), addr.port())
                    } else {
                        format!("http://{}:{}", addr.ip(), addr.port())
                    };

                    tracing::info!(url, "Listening on TCP");
                    axum::serve(listener, app)
                        .await
                        .expect("Server error on TCP listener");
                }
                ListenAddr::Unix(path) => {
                    // Remove existing socket file if it exists
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

    // Wait for all listeners (this will run forever unless interrupted)
    for task in tasks {
        task.await.expect("Listener task panicked");
    }
}

/// Shared application state
#[derive(Clone)]
struct AppState {
    downstream_url: String,
}

/// Custom error type for proxy operations
#[derive(Debug)]
enum ProxyError {
    /// Network error (connection failed, timeout, etc.)
    Network(reqwest::Error),
}

impl std::fmt::Display for ProxyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyError::Network(e) => write!(f, "Network error: {}", e),
        }
    }
}

impl std::error::Error for ProxyError {}

/// Check if a path represents a static asset that should be logged at TRACE level
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

/// Check if a path represents a page route (heuristic: no file extension)
fn is_page_route(path: &str) -> bool {
    !path.starts_with("/node_modules/")
        && !path.starts_with("/@")
        && !path.starts_with("/.svelte-kit/")
        && !path.contains('.') // Simple heuristic: no extension = likely a page
}

// API routes for data endpoints
fn api_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(health_handler))
        .route("/projects", get(projects_handler))
}

// Health check endpoint
async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

// API 404 fallback handler - catches unmatched /api/* routes
async fn api_404_handler(uri: axum::http::Uri) -> impl IntoResponse {
    tracing::warn!(path = %uri.path(), "API route not found");
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "error": "Not found",
            "path": uri.path()
        })),
    )
}

// Project data structure
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

// Projects endpoint - returns hardcoded project data for now
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

// ISR handler - proxies to Bun SSR server
// This is the fallback for all routes not matched by /api/*
#[tracing::instrument(skip(state, req), fields(path = %req.uri().path()))]
async fn isr_handler(State(state): State<Arc<AppState>>, req: Request) -> Response {
    let uri = req.uri();
    let path = uri.path();
    let query = uri.query().unwrap_or("");

    // Check if API route somehow reached ISR handler (shouldn't happen)
    if path.starts_with("/api/") {
        tracing::error!("API request reached ISR handler - routing bug!");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal routing error",
        )
            .into_response();
    }

    // Build URL for Bun server
    let bun_url = if query.is_empty() {
        format!("{}{}", state.downstream_url, path)
    } else {
        format!("{}{}?{}", state.downstream_url, path, query)
    };

    // Track request timing
    let start = std::time::Instant::now();

    // TODO: Add ISR caching layer here (moka, singleflight, stale-while-revalidate)
    // For now, just proxy directly to Bun

    match proxy_to_bun(&bun_url, &state.downstream_url).await {
        Ok((status, headers, body)) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let cache = "miss"; // Hardcoded for now, will change when caching is implemented

            // Intelligent logging based on path type and status
            let is_static = is_static_asset(path);
            let is_page = is_page_route(path);

            match (status.as_u16(), is_static, is_page) {
                // Static assets - success at TRACE
                (200..=299, true, _) => {
                    tracing::trace!(status = status.as_u16(), duration_ms, cache, "ISR request");
                }
                // Static assets - 404 at WARN
                (404, true, _) => {
                    tracing::warn!(
                        status = status.as_u16(),
                        duration_ms,
                        cache,
                        "ISR request - missing asset"
                    );
                }
                // Static assets - server error at ERROR
                (500..=599, true, _) => {
                    tracing::error!(
                        status = status.as_u16(),
                        duration_ms,
                        cache,
                        "ISR request - server error"
                    );
                }
                // Page routes - success at DEBUG
                (200..=299, _, true) => {
                    tracing::debug!(status = status.as_u16(), duration_ms, cache, "ISR request");
                }
                // Page routes - 404 silent (normal case for non-existent pages)
                (404, _, true) => {}
                // Page routes - server error at ERROR
                (500..=599, _, _) => {
                    tracing::error!(
                        status = status.as_u16(),
                        duration_ms,
                        cache,
                        "ISR request - server error"
                    );
                }
                // Default fallback - DEBUG
                _ => {
                    tracing::debug!(status = status.as_u16(), duration_ms, cache, "ISR request");
                }
            }

            // Forward response
            (status, headers, body).into_response()
        }
        Err(err) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            tracing::error!(
                error = %err,
                url = %bun_url,
                duration_ms,
                "Failed to proxy to Bun"
            );
            (
                StatusCode::BAD_GATEWAY,
                format!("Failed to render page: {}", err),
            )
                .into_response()
        }
    }
}

// Proxy a request to the Bun SSR server, returning status, headers and body
async fn proxy_to_bun(
    url: &str,
    downstream_url: &str,
) -> Result<(StatusCode, HeaderMap, String), ProxyError> {
    // Check if downstream is a Unix socket path
    let client = if downstream_url.starts_with('/') || downstream_url.starts_with("./") {
        // Unix socket
        let path = PathBuf::from(downstream_url);
        reqwest::Client::builder()
            .unix_socket(path)
            .build()
            .map_err(ProxyError::Network)?
    } else {
        // Regular HTTP
        reqwest::Client::new()
    };

    let response = client.get(url).send().await.map_err(ProxyError::Network)?;

    // Extract status code
    let status = StatusCode::from_u16(response.status().as_u16())
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    // Convert reqwest headers to axum HeaderMap
    let mut headers = HeaderMap::new();
    for (name, value) in response.headers() {
        // Skip hop-by-hop headers and content-length (axum will recalculate it)
        let name_str = name.as_str();
        if name_str == "transfer-encoding"
            || name_str == "connection"
            || name_str == "content-length"
        {
            continue;
        }

        if let Ok(header_name) = axum::http::HeaderName::try_from(name.as_str()) {
            if let Ok(header_value) = axum::http::HeaderValue::try_from(value.as_bytes()) {
                headers.insert(header_name, header_value);
            }
        }
    }

    let body = response.text().await.map_err(ProxyError::Network)?;
    Ok((status, headers, body))
}
