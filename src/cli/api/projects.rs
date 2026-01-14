use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::cli::client::{ApiClient, ApiError, check_response};
use crate::cli::output;
use crate::cli::{ProjectsCommand, TagOp, parse_create_tags, parse_update_tags};
use crate::db::{ApiAdminProject, ApiTag, ProjectStatus};

/// Request to create a project
#[derive(Serialize)]
struct CreateProjectRequest {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    slug: Option<String>,
    short_description: String,
    description: String,
    status: ProjectStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    github_repo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    demo_url: Option<String>,
    tag_ids: Vec<String>,
}

/// Request to update a project
#[derive(Serialize)]
struct UpdateProjectRequest {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    slug: Option<String>,
    short_description: String,
    description: String,
    status: ProjectStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    github_repo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    demo_url: Option<String>,
    tag_ids: Vec<String>,
}

/// Run a projects subcommand
pub async fn run(
    client: ApiClient,
    command: ProjectsCommand,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        ProjectsCommand::List => list(client, json).await,
        ProjectsCommand::Get { reference } => get(client, &reference, json).await,
        ProjectsCommand::Create {
            name,
            short_desc,
            desc,
            slug,
            status,
            github_repo,
            demo_url,
            tags,
        } => {
            let tag_slugs = tags.map(|s| parse_create_tags(&s)).unwrap_or_default();
            create(
                client,
                &name,
                &short_desc,
                &desc,
                slug,
                &status,
                github_repo,
                demo_url,
                tag_slugs,
                json,
            )
            .await
        }
        ProjectsCommand::Update {
            reference,
            name,
            slug,
            short_desc,
            desc,
            status,
            github_repo,
            demo_url,
            tags,
        } => {
            let tag_ops = match tags {
                Some(s) => parse_update_tags(&s)?,
                None => vec![],
            };
            update(
                client,
                &reference,
                name,
                slug,
                short_desc,
                desc,
                status,
                github_repo,
                demo_url,
                tag_ops,
                json,
            )
            .await
        }
        ProjectsCommand::Delete { reference } => delete(client, &reference, json).await,
    }
}

/// List all projects
async fn list(client: ApiClient, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let response = client.get_auth("/api/projects").await?;
    let response = check_response(response).await?;
    let projects: Vec<ApiAdminProject> = response.json().await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&projects)?);
    } else {
        output::print_projects_table(&projects);
    }

    Ok(())
}

/// Get a project by slug or UUID
async fn get(
    client: ApiClient,
    reference: &str,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let project = resolve_project(&client, reference).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&project)?);
    } else {
        output::print_project(&project);
    }

    Ok(())
}

/// Create a new project
#[allow(clippy::too_many_arguments)]
async fn create(
    client: ApiClient,
    name: &str,
    short_desc: &str,
    desc: &str,
    slug: Option<String>,
    status: &str,
    github_repo: Option<String>,
    demo_url: Option<String>,
    tag_slugs: Vec<String>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Resolve tag slugs to IDs
    let tag_ids = resolve_tag_ids(&client, &tag_slugs).await?;

    let status = parse_status(status)?;

    let request = CreateProjectRequest {
        name: name.to_string(),
        slug,
        short_description: short_desc.to_string(),
        description: desc.to_string(),
        status,
        github_repo: github_repo.filter(|s| !s.is_empty()),
        demo_url: demo_url.filter(|s| !s.is_empty()),
        tag_ids,
    };

    let response = client.post_auth("/api/projects", &request).await?;
    let response = check_response(response).await?;
    let project: ApiAdminProject = response.json().await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&project)?);
    } else {
        output::success(&format!("Created project: {}", project.project.name));
        output::print_project(&project);
    }

    Ok(())
}

/// Update an existing project
#[allow(clippy::too_many_arguments)]
async fn update(
    client: ApiClient,
    reference: &str,
    name: Option<String>,
    slug: Option<String>,
    short_desc: Option<String>,
    desc: Option<String>,
    status: Option<String>,
    github_repo: Option<String>,
    demo_url: Option<String>,
    tag_ops: Vec<TagOp>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // First fetch the current project
    let current = resolve_project(&client, reference).await?;

    // Apply tag operations
    let mut current_tag_ids: Vec<String> = current.tags.iter().map(|t| t.id.clone()).collect();

    for op in tag_ops {
        match op {
            TagOp::Add(slug_or_id) => {
                let tag_id = resolve_tag_id(&client, &slug_or_id).await?;
                if !current_tag_ids.contains(&tag_id) {
                    current_tag_ids.push(tag_id);
                }
            }
            TagOp::Remove(slug_or_id) => {
                let tag_id = resolve_tag_id(&client, &slug_or_id).await?;
                current_tag_ids.retain(|id| id != &tag_id);
            }
        }
    }

    // Merge updates with current values
    let status = if let Some(s) = status {
        parse_status(&s)?
    } else {
        parse_status(&current.status)?
    };

    let request = UpdateProjectRequest {
        name: name.unwrap_or(current.project.name),
        slug,
        short_description: short_desc.unwrap_or(current.project.short_description),
        description: desc.unwrap_or(current.description),
        status,
        github_repo: match github_repo {
            Some(s) if s.is_empty() => None,
            Some(s) => Some(s),
            None => current.github_repo,
        },
        demo_url: match demo_url {
            Some(s) if s.is_empty() => None,
            Some(s) => Some(s),
            None => current.demo_url,
        },
        tag_ids: current_tag_ids,
    };

    let response = client
        .put_auth(&format!("/api/projects/{}", current.project.id), &request)
        .await?;
    let response = check_response(response).await?;
    let project: ApiAdminProject = response.json().await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&project)?);
    } else {
        output::success(&format!("Updated project: {}", project.project.name));
        output::print_project(&project);
    }

    Ok(())
}

/// Delete a project
async fn delete(
    client: ApiClient,
    reference: &str,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // First resolve to get the ID
    let project = resolve_project(&client, reference).await?;

    let response = client
        .delete_auth(&format!("/api/projects/{}", project.project.id))
        .await?;
    let response = check_response(response).await?;
    let deleted: ApiAdminProject = response.json().await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&deleted)?);
    } else {
        output::success(&format!("Deleted project: {}", deleted.project.name));
    }

    Ok(())
}

/// Resolve a project reference (slug or UUID) to a full project
async fn resolve_project(
    client: &ApiClient,
    reference: &str,
) -> Result<ApiAdminProject, Box<dyn std::error::Error>> {
    // Try as UUID first
    if Uuid::parse_str(reference).is_ok() {
        let response = client
            .get_auth(&format!("/api/projects/{}", reference))
            .await?;
        let response = check_response(response).await?;
        return Ok(response.json().await?);
    }

    // Otherwise search by slug in the list
    let response = client.get_auth("/api/projects").await?;
    let response = check_response(response).await?;
    let projects: Vec<ApiAdminProject> = response.json().await?;

    projects
        .into_iter()
        .find(|p| p.project.slug == reference)
        .ok_or_else(|| ApiError::Parse(format!("Project not found: {}", reference)).into())
}

/// Resolve tag slugs to IDs
async fn resolve_tag_ids(
    client: &ApiClient,
    slugs: &[String],
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    if slugs.is_empty() {
        return Ok(vec![]);
    }

    let mut ids = Vec::new();
    for slug in slugs {
        ids.push(resolve_tag_id(client, slug).await?);
    }
    Ok(ids)
}

/// Resolve a single tag slug or ID to an ID
async fn resolve_tag_id(
    client: &ApiClient,
    slug_or_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Try as UUID first
    if Uuid::parse_str(slug_or_id).is_ok() {
        return Ok(slug_or_id.to_string());
    }

    // Otherwise look up by slug
    #[derive(Deserialize)]
    struct TagResponse {
        tag: ApiTag,
    }

    let response = client.get(&format!("/api/tags/{}", slug_or_id)).await?;
    let response = check_response(response).await?;
    let tag_response: TagResponse = response.json().await?;
    Ok(tag_response.tag.id)
}

/// Parse status string to ProjectStatus
fn parse_status(s: &str) -> Result<ProjectStatus, Box<dyn std::error::Error>> {
    match s.to_lowercase().as_str() {
        "active" => Ok(ProjectStatus::Active),
        "maintained" => Ok(ProjectStatus::Maintained),
        "archived" => Ok(ProjectStatus::Archived),
        "hidden" => Ok(ProjectStatus::Hidden),
        _ => Err(format!(
            "Invalid status '{}'. Valid values: active, maintained, archived, hidden",
            s
        )
        .into()),
    }
}
