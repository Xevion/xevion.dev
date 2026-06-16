use crate::cli::SettingsCommand;
use crate::cli::client::{ApiClient, check_response, json as decode_json};
use crate::cli::error::CliError;
use crate::cli::output;
use crate::db::{ApiSiteSettings, UpdateSiteIdentityRequest, UpdateSiteSettingsRequest};

/// Run a settings subcommand
pub async fn run(client: ApiClient, command: SettingsCommand, json: bool) -> Result<(), CliError> {
    match command {
        SettingsCommand::Get => get(client, json).await,
        SettingsCommand::Update {
            display_name,
            occupation,
            bio,
            site_title,
        } => update(client, display_name, occupation, bio, site_title, json).await,
    }
}

/// Get current site settings
async fn get(client: ApiClient, json: bool) -> Result<(), CliError> {
    let settings: ApiSiteSettings =
        decode_json(check_response(client.get("/api/settings").await?).await?).await?;

    if json {
        output::print_json(&settings)?;
    } else {
        output::print_settings(&settings);
    }

    Ok(())
}

/// Update site settings
async fn update(
    client: ApiClient,
    display_name: Option<String>,
    occupation: Option<String>,
    bio: Option<String>,
    site_title: Option<String>,
    json: bool,
) -> Result<(), CliError> {
    // First fetch current settings
    let current: ApiSiteSettings =
        decode_json(check_response(client.get("/api/settings").await?).await?).await?;

    // Merge updates
    let request = UpdateSiteSettingsRequest {
        identity: UpdateSiteIdentityRequest {
            display_name: display_name.unwrap_or(current.identity.display_name),
            occupation: occupation.unwrap_or(current.identity.occupation),
            bio: bio.unwrap_or(current.identity.bio),
            site_title: site_title.unwrap_or(current.identity.site_title),
        },
        // Keep existing social links unchanged
        social_links: current
            .social_links
            .into_iter()
            .map(|link| crate::db::UpdateSocialLinkRequest {
                id: link.id,
                platform: link.platform,
                label: link.label,
                value: link.value,
                icon: link.icon,
                visible: link.visible,
                display_order: link.display_order,
            })
            .collect(),
    };

    let settings: ApiSiteSettings =
        decode_json(check_response(client.put("/api/settings", &request).await?).await?).await?;

    if json {
        output::print_json(&settings)?;
    } else {
        output::success("Updated site settings");
        output::print_settings(&settings);
    }

    Ok(())
}
