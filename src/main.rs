use clap::Parser;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod assets;
mod auth;
mod cache;
mod cli;
mod config;
mod db;
mod formatter;
mod github;
mod handlers;
mod health;
mod http;
mod media_processing;
mod middleware;
mod og;
mod proxy;
mod r2;
mod routes;
mod state;
mod tarpit;
mod utils;

use cli::{Cli, Command};
use formatter::{CustomJsonFormatter, CustomPrettyFormatter};

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
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Some(Command::Api(api_args)) => {
            // API client commands - no tracing needed
            if let Err(e) = cli::api::run(api_args).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        None => {
            // No subcommand - run the server
            init_tracing();

            // Validate required server args
            if args.listen.is_empty() {
                eprintln!("Error: --listen is required when running the server");
                eprintln!("Example: xevion --listen :8080 --downstream http://localhost:5173");
                std::process::exit(1);
            }

            let downstream = match args.downstream {
                Some(d) => d,
                None => {
                    eprintln!("Error: --downstream is required when running the server");
                    eprintln!("Example: xevion --listen :8080 --downstream http://localhost:5173");
                    std::process::exit(1);
                }
            };

            if let Err(e) = cli::serve::run(args.listen, downstream, args.trust_request_id).await {
                eprintln!("Server error: {}", e);
                std::process::exit(1);
            }
        }
    }
}
