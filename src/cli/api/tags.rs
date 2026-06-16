use serde::Deserialize;

use crate::cli::TagsCommand;
use crate::cli::client::{ApiClient, check_response, json as decode_json};
use crate::cli::error::CliError;
use crate::cli::output;
use crate::db::{ApiTag, ApiTagWithCount};
use crate::handlers::{CreateTagRequest, UpdateTagRequest};

/// Response for get tag endpoint
#[derive(Deserialize)]
struct GetTagResponse {
    tag: ApiTag,
    projects: Vec<serde_json::Value>,
}

/// Run a tags subcommand
pub async fn run(client: ApiClient, command: TagsCommand, json: bool) -> Result<(), CliError> {
    match command {
        TagsCommand::List => list(client, json).await,
        TagsCommand::Get { reference } => get(client, &reference, json).await,
        TagsCommand::Create {
            name,
            slug,
            icon,
            color,
        } => create(client, &name, slug, icon, color, json).await,
        TagsCommand::Update {
            reference,
            name,
            slug,
            icon,
            color,
        } => update(client, &reference, name, slug, icon, color, json).await,
        TagsCommand::Delete { reference } => delete(client, &reference, json).await,
    }
}

/// List all tags
async fn list(client: ApiClient, json: bool) -> Result<(), CliError> {
    let tags: Vec<ApiTagWithCount> =
        decode_json(check_response(client.get("/api/tags").await?).await?).await?;

    if json {
        output::print_json(&tags)?;
    } else {
        output::print_tags_table(&tags);
    }

    Ok(())
}

/// Get a tag by slug or UUID
async fn get(client: ApiClient, reference: &str, json: bool) -> Result<(), CliError> {
    let tag_response: GetTagResponse =
        decode_json(check_response(client.get(&format!("/api/tags/{reference}")).await?).await?)
            .await?;

    if json {
        output::print_json(&serde_json::json!({
            "tag": tag_response.tag,
            "projects": tag_response.projects,
        }))?;
    } else {
        output::print_tag(&tag_response.tag);
        if !tag_response.projects.is_empty() {
            output::info(&format!(
                "{} associated project(s)",
                tag_response.projects.len()
            ));
        }
    }

    Ok(())
}

/// Create a new tag. `color` is already normalized to bare hex by the arg parser.
async fn create(
    client: ApiClient,
    name: &str,
    slug: Option<String>,
    icon: Option<String>,
    color: Option<String>,
    json: bool,
) -> Result<(), CliError> {
    let request = CreateTagRequest {
        name: name.to_string(),
        slug,
        icon: icon.filter(|s| !s.is_empty()),
        color: color.filter(|s| !s.is_empty()),
    };

    let tag: ApiTag =
        decode_json(check_response(client.post("/api/tags", &request).await?).await?).await?;

    if json {
        output::print_json(&tag)?;
    } else {
        output::success(&format!("Created tag: {}", tag.name));
        output::print_tag(&tag);
    }

    Ok(())
}

/// Update an existing tag. `color` is already normalized to bare hex by the arg parser.
async fn update(
    client: ApiClient,
    reference: &str,
    name: Option<String>,
    new_slug: Option<String>,
    icon: Option<String>,
    color: Option<String>,
    json: bool,
) -> Result<(), CliError> {
    // First fetch the current tag
    let current: GetTagResponse =
        decode_json(check_response(client.get(&format!("/api/tags/{reference}")).await?).await?)
            .await?;

    // Merge updates
    let request = UpdateTagRequest {
        name: name.unwrap_or(current.tag.name),
        slug: new_slug,
        icon: match icon {
            Some(s) if s.is_empty() => None,
            Some(s) => Some(s),
            None => current.tag.icon,
        },
        color: match color {
            Some(s) if s.is_empty() => None,
            Some(s) => Some(s),
            None => current.tag.color,
        },
    };

    let tag: ApiTag = decode_json(
        check_response(
            client
                .put(&format!("/api/tags/{reference}"), &request)
                .await?,
        )
        .await?,
    )
    .await?;

    if json {
        output::print_json(&tag)?;
    } else {
        output::success(&format!("Updated tag: {}", tag.name));
        output::print_tag(&tag);
    }

    Ok(())
}

/// Delete a tag
async fn delete(client: ApiClient, reference: &str, json: bool) -> Result<(), CliError> {
    let deleted: ApiTag =
        decode_json(check_response(client.delete(&format!("/api/tags/{reference}")).await?).await?)
            .await?;

    if json {
        output::print_json(&deleted)?;
    } else {
        output::success(&format!("Deleted tag: {}", deleted.name));
    }

    Ok(())
}
