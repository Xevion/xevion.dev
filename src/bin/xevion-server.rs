//! `xevion-server` — the web server (reverse proxy, API, asset serving) plus the
//! local database `seed` command. The remote content-management CLI lives in the
//! `xevion` binary.

use clap::{Parser, Subcommand};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use api::config::ListenAddr;
use api::formatter::{CustomJsonFormatter, CustomPrettyFormatter};
use api::{cli, db};

/// xevion.dev — personal portfolio web server.
#[derive(Parser, Debug)]
#[command(name = "xevion-server")]
#[command(about = "Personal portfolio web server (reverse proxy, API, SSR proxy)")]
#[command(version)]
struct ServerCli {
    #[command(subcommand)]
    command: Option<ServerCommand>,

    /// Address(es) to listen on (TCP or Unix socket)
    #[arg(long, env = "LISTEN_ADDR", value_delimiter = ',')]
    listen: Vec<ListenAddr>,

    /// Downstream SSR server URL
    #[arg(long, env = "DOWNSTREAM_URL")]
    downstream: Option<String>,

    /// Trust X-Request-ID header from specified source
    #[arg(long, env = "TRUST_REQUEST_ID")]
    trust_request_id: Option<String>,
}

#[derive(Subcommand, Debug)]
enum ServerCommand {
    /// Seed the database with sample data
    Seed,
}

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
    let args = ServerCli::parse();

    match args.command {
        Some(ServerCommand::Seed) => {
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
        None => {
            // No subcommand - run the server
            init_tracing();

            // Resolve ports: PORT defaults to 10237, FRONTEND_PORT defaults to PORT+1
            let default_port: u16 = std::env::var("PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10237);

            let listen = if args.listen.is_empty() {
                vec![ListenAddr::Tcp(std::net::SocketAddr::from((
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
