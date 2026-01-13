use clap::Parser;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod assets;
mod auth;
mod cache;
mod config;
mod db;
mod formatter;
mod handlers;
mod health;
mod http;
mod middleware;
mod og;
mod proxy;
mod r2;
mod routes;
mod state;
mod tarpit;
mod utils;

use cache::{IsrCache, IsrCacheConfig};
use config::{Args, ListenAddr};
use formatter::{CustomJsonFormatter, CustomPrettyFormatter};
use health::HealthChecker;
use middleware::RequestIdLayer;
use state::AppState;
use tarpit::{TarpitConfig, TarpitState};

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

    // Check and run migrations on startup
    let migrator = sqlx::migrate!();

    // Query applied migrations directly from the database
    let applied_versions: HashSet<i64> =
        sqlx::query_scalar::<_, i64>("SELECT version FROM _sqlx_migrations ORDER BY version")
            .fetch_all(&pool)
            .await
            .unwrap_or_default()
            .into_iter()
            .collect();

    let pending: Vec<_> = migrator
        .iter()
        .filter(|m| !m.migration_type.is_down_migration())
        .filter(|m| !applied_versions.contains(&m.version))
        .map(|m| m.description.as_ref())
        .collect();

    if pending.is_empty() {
        let last_version = applied_versions.iter().max();
        let last_name = last_version
            .and_then(|v| migrator.iter().find(|m| m.version == *v))
            .map(|m| m.description.as_ref());
        tracing::debug!(last_migration = ?last_name, "Database schema is current");
    } else {
        tracing::warn!(migrations = ?pending, "Pending database migrations");
    }

    migrator.run(&pool).await.unwrap_or_else(|e| {
        tracing::error!(error = %e, "Migration failed");
        std::process::exit(1);
    });

    if !pending.is_empty() {
        tracing::info!(count = pending.len(), "Migrations applied");
    }

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

    // Create socket-aware HTTP client
    let client = http::HttpClient::new(&args.downstream).expect("Failed to create HTTP client");

    // Create health checker
    let client_for_health = client.clone();
    let pool_for_health = pool.clone();

    let health_checker = Arc::new(HealthChecker::new(move || {
        let client = client_for_health.clone();
        let pool = pool_for_health.clone();

        async move { proxy::perform_health_check(client, Some(pool)).await }
    }));

    let tarpit_config = TarpitConfig::from_env();
    let tarpit_state = Arc::new(TarpitState::new(tarpit_config));

    tracing::info!(
        enabled = tarpit_state.config.enabled,
        delay_min_ms = tarpit_state.config.delay_min_ms,
        delay_max_ms = tarpit_state.config.delay_max_ms,
        max_global = tarpit_state.config.max_global_connections,
        max_per_ip = tarpit_state.config.max_connections_per_ip,
        "Tarpit initialized"
    );

    // Initialize ISR cache
    let isr_cache_config = IsrCacheConfig::from_env();
    let isr_cache = Arc::new(IsrCache::new(isr_cache_config.clone()));

    tracing::info!(
        enabled = isr_cache_config.enabled,
        max_entries = isr_cache_config.max_entries,
        fresh_sec = isr_cache_config.fresh_duration.as_secs(),
        stale_sec = isr_cache_config.stale_duration.as_secs(),
        "ISR cache initialized"
    );

    let state = Arc::new(AppState {
        client,
        health_checker,
        tarpit_state,
        pool: pool.clone(),
        session_manager: session_manager.clone(),
        isr_cache,
    });

    // Regenerate common OGP images on startup
    tokio::spawn({
        let state = state.clone();
        async move {
            og::regenerate_common_images(state).await;
        }
    });

    // Apply middleware to router
    fn apply_middleware(
        router: axum::Router<Arc<AppState>>,
        trust_request_id: Option<String>,
    ) -> axum::Router<Arc<AppState>> {
        router
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
                        routes::build_base_router().fallback(proxy::fallback_handler_tcp),
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
                        routes::build_base_router().fallback(proxy::fallback_handler_unix),
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
