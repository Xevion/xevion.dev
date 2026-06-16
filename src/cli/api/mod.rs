pub mod auth;
pub mod content;
pub mod projects;
pub mod settings;
pub mod tags;

use crate::cli::client::ApiClient;
use crate::cli::config::Config;
use crate::cli::error::CliError;
use crate::cli::{ApiArgs, ApiCommand, ConfigCommand};

/// Run an API subcommand. Loads the config, resolves the target, and dispatches.
pub async fn run(args: ApiArgs) -> Result<(), CliError> {
    let config_path = crate::cli::config::resolve_path(args.config.as_deref())?;
    let config = Config::load(config_path)?;
    let api = args.api.as_deref();
    let json = args.json;

    match args.command {
        // Login manages the config itself and doesn't need a pre-resolved target.
        ApiCommand::Login {
            url,
            label,
            no_browser,
        } => auth::login(config, api, url, label, no_browser, json).await,

        ApiCommand::Targets { command } => auth::targets(config, command, json),

        ApiCommand::Config(ConfigCommand::Path) => {
            auth::config_path(&config, json);
            Ok(())
        }

        ApiCommand::Logout => {
            let (name, entry) = config.resolve(api)?;
            let client = ApiClient::new(entry.url, entry.token);
            auth::logout(config, name, client, json).await
        }

        ApiCommand::Session => {
            let (name, entry) = config.resolve(api)?;
            let client = ApiClient::new(entry.url, entry.token);
            auth::session(client, &name, json).await
        }

        // Data commands all need an authenticated client.
        ApiCommand::Projects(cmd) => projects::run(authed_client(&config, api)?, cmd, json).await,
        ApiCommand::Tags(cmd) => tags::run(authed_client(&config, api)?, cmd, json).await,
        ApiCommand::Settings(cmd) => settings::run(authed_client(&config, api)?, cmd, json).await,
    }
}

/// Resolve the selected target and build an authenticated client. `config.resolve`
/// guarantees a token, so the resulting client is always authorized — there is no
/// second client-side auth check.
pub fn authed_client(config: &Config, api: Option<&str>) -> Result<ApiClient, CliError> {
    let (_, entry) = config.resolve(api)?;
    Ok(ApiClient::new(entry.url, entry.token))
}
