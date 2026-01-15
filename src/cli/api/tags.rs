use serde::Deserialize;

use crate::cli::TagsCommand;
use crate::cli::client::{ApiClient, check_response};
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
pub async fn run(
    client: ApiClient,
    command: TagsCommand,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        TagsCommand::List => list(client, json).await,
        TagsCommand::Get { slug } => get(client, &slug, json).await,
        TagsCommand::Create {
            name,
            slug,
            icon,
            color,
        } => create(client, &name, slug, icon, color, json).await,
        TagsCommand::Update {
            slug,
            name,
            new_slug,
            icon,
            color,
        } => update(client, &slug, name, new_slug, icon, color, json).await,
        TagsCommand::Delete { reference } => delete(client, &reference, json).await,
    }
}

/// List all tags
async fn list(client: ApiClient, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let response = client.get("/api/tags").await?;
    let response = check_response(response).await?;
    let tags: Vec<ApiTagWithCount> = response.json().await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&tags)?);
    } else {
        output::print_tags_table(&tags);
    }

    Ok(())
}

/// Get a tag by slug
async fn get(client: ApiClient, slug: &str, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let response = client.get(&format!("/api/tags/{}", slug)).await?;
    let response = check_response(response).await?;
    let tag_response: GetTagResponse = response.json().await?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "tag": tag_response.tag,
                "projects": tag_response.projects,
            }))?
        );
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

/// Create a new tag
async fn create(
    client: ApiClient,
    name: &str,
    slug: Option<String>,
    icon: Option<String>,
    color: Option<String>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Validate color if provided
    if let Some(ref c) = color
        && (!c.chars().all(|ch| ch.is_ascii_hexdigit()) || c.len() != 6)
    {
        return Err("Color must be a 6-character hex string (e.g., '3b82f6')".into());
    }

    let request = CreateTagRequest {
        name: name.to_string(),
        slug,
        icon: icon.filter(|s| !s.is_empty()),
        color: color.filter(|s| !s.is_empty()),
    };

    let response = client.post_auth("/api/tags", &request).await?;
    let response = check_response(response).await?;
    let tag: ApiTag = response.json().await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&tag)?);
    } else {
        output::success(&format!("Created tag: {}", tag.name));
        output::print_tag(&tag);
    }

    Ok(())
}

/// Update an existing tag
async fn update(
    client: ApiClient,
    slug: &str,
    name: Option<String>,
    new_slug: Option<String>,
    icon: Option<String>,
    color: Option<String>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Validate color if provided
    if let Some(ref c) = color
        && !c.is_empty()
        && (!c.chars().all(|ch| ch.is_ascii_hexdigit()) || c.len() != 6)
    {
        return Err("Color must be a 6-character hex string (e.g., '3b82f6')".into());
    }

    // First fetch the current tag
    let response = client.get(&format!("/api/tags/{}", slug)).await?;
    let response = check_response(response).await?;
    let current: GetTagResponse = response.json().await?;

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

    let response = client
        .put_auth(&format!("/api/tags/{}", slug), &request)
        .await?;
    let response = check_response(response).await?;
    let tag: ApiTag = response.json().await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&tag)?);
    } else {
        output::success(&format!("Updated tag: {}", tag.name));
        output::print_tag(&tag);
    }

    Ok(())
}

/// Delete a tag
async fn delete(
    client: ApiClient,
    reference: &str,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = client
        .delete_auth(&format!("/api/tags/{}", reference))
        .await?;
    let response = check_response(response).await?;
    let deleted: ApiTag = response.json().await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&deleted)?);
    } else {
        output::success(&format!("Deleted tag: {}", deleted.name));
    }

    Ok(())
}
