pub mod auth;
pub mod content;
pub mod projects;
pub mod settings;
pub mod tags;

use crate::cli::client::ApiClient;
use crate::cli::config::Config;
use crate::cli::{ApiArgs, ApiCommand};

/// Run an API subcommand. Loads the config, resolves the target, and dispatches.
pub async fn run(args: ApiArgs) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = crate::cli::config::resolve_path(args.config.as_deref());
    let config = Config::load(config_path)?;
    let api = args.api.as_deref();

    match args.command {
        // Login manages the config itself and doesn't need a pre-resolved target.
        ApiCommand::Login {
            url,
            label,
            no_browser,
        } => auth::login(config, api, url, label, no_browser).await,

        ApiCommand::Targets => {
            auth::targets(&config, args.json);
            Ok(())
        }

        ApiCommand::Logout => {
            let (name, entry) = config.resolve(api)?;
            let client = ApiClient::new(entry.url, entry.token);
            auth::logout(config, name, client).await
        }

        ApiCommand::Session => {
            let (name, entry) = config.resolve(api)?;
            let client = ApiClient::new(entry.url, entry.token);
            auth::session(client, &name, args.json).await
        }

        // Data commands all need an authenticated client.
        ApiCommand::Projects(cmd) => {
            projects::run(authed_client(&config, api)?, cmd, args.json).await
        }
        ApiCommand::Tags(cmd) => tags::run(authed_client(&config, api)?, cmd, args.json).await,
        ApiCommand::Settings(cmd) => {
            settings::run(authed_client(&config, api)?, cmd, args.json).await
        }
    }
}

/// Resolve the selected target and build an authenticated client.
fn authed_client(
    config: &Config,
    api: Option<&str>,
) -> Result<ApiClient, Box<dyn std::error::Error>> {
    let (_, entry) = config.resolve(api)?;
    Ok(ApiClient::new(entry.url, entry.token))
}
