pub mod auth;
pub mod projects;
pub mod settings;
pub mod tags;

use crate::cli::client::ApiClient;
use crate::cli::{ApiArgs, ApiCommand};

/// Run an API subcommand
pub async fn run(args: ApiArgs) -> Result<(), Box<dyn std::error::Error>> {
    let client = ApiClient::new(args.api_url, args.session);

    match args.command {
        ApiCommand::Login { username, password } => auth::login(client, &username, &password).await,
        ApiCommand::Logout => auth::logout(client).await,
        ApiCommand::Session => auth::session(client, args.json).await,
        ApiCommand::Projects(cmd) => projects::run(client, cmd, args.json).await,
        ApiCommand::Tags(cmd) => tags::run(client, cmd, args.json).await,
        ApiCommand::Settings(cmd) => settings::run(client, cmd, args.json).await,
    }
}
