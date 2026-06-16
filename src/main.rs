use clap::Parser;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod assets;
mod auth;
mod cache;
mod cli;
mod cli_auth;
mod config;
mod db;
mod encoding;
mod events;
mod formatter;
mod github;
mod handlers;
mod health;
mod http;
mod icon_cache;
mod markdown;
mod media_processing;
mod middleware;
mod og;
mod pm;
mod proxy;
mod r2;
mod routes;
mod state;
mod tarpit;
mod utils;

use cli::{Cli, Command};
use formatter::{CustomJsonFormatter, CustomPrettyFormatter};

fn init_tracing() {
    let use_json = std::env::var("LOG_JSON").is_ok_and(|v| v == "true" || v == "1");

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
    // Install ring as the default TLS crypto provider (must happen before any TLS usage)
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install ring crypto provider");

    // Load .env file if present
    dotenvy::dotenv().ok();

    // Parse args early to allow --help to work without database
    let args = Cli::parse();

    match args.command {
        Some(Command::Seed) => {
            // Seed command - connect to database and run seeding
            let database_url =
                std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in environment");

            let pool = db::create_pool(&database_url)
                .await
                .expect("Failed to connect to database");

            // Run migrations first
            sqlx::migrate!()
                .run(&pool)
                .await
                .expect("Failed to run migrations");

            if let Err(e) = cli::seed::run(&pool).await {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
        Some(Command::Api(api_args)) => {
            // API client commands - no tracing needed
            let json = api_args.json;
            if let Err(e) = cli::api::run(*api_args).await {
                let code = e.exit_code();
                if json {
                    // Machine-readable error on stdout for scripted callers.
                    println!("{}", e.to_json());
                } else {
                    // Rich diagnostic (help line + source chain) on stderr.
                    eprintln!("{:?}", miette::Report::new(e));
                }
                std::process::exit(code);
            }
        }
        None => {
            // No subcommand - run the server
            init_tracing();

            // Resolve ports: PORT defaults to 10237, FRONTEND_PORT defaults to PORT+1
            let default_port: u16 = std::env::var("PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10237);

            let listen = if args.listen.is_empty() {
                vec![config::ListenAddr::Tcp(std::net::SocketAddr::from((
                    [127, 0, 0, 1],
                    default_port,
                )))]
            } else {
                args.listen
            };

            let downstream = args.downstream.unwrap_or_else(|| {
                let frontend_port: u16 = std::env::var("FRONTEND_PORT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(|| default_port + 1);
                format!("http://localhost:{frontend_port}")
            });

            if let Err(e) = cli::serve::run(listen, downstream, args.trust_request_id).await {
                eprintln!("Server error: {e}");
                std::process::exit(1);
            }
        }
    }
}
