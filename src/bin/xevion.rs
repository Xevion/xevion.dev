//! `xevion` — the API client CLI for managing xevion.dev content remotely.
//!
//! The `api` subcommands are hoisted to the top level here (`xevion projects
//! list`, `xevion login`, …). The web server lives in the `xevion-server`
//! binary.

use clap::Parser;

use api::cli::ApiArgs;

#[tokio::main]
async fn main() {
    // Install ring as the default TLS crypto provider (reqwest uses rustls with
    // no bundled provider). Must happen before any TLS usage.
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install ring crypto provider");

    // Load .env if present (lets local runs pick up XEVION_API / XEVION_CONFIG).
    dotenvy::dotenv().ok();

    let args = ApiArgs::parse();
    let json = args.json;

    if let Err(e) = api::cli::api::run(args).await {
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
