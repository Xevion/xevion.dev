use uuid::Uuid;

use crate::cli::client::{ApiClient, check_response, json as decode_json};
use crate::cli::error::CliError;
use crate::cli::output;
use crate::cli::{ProjectsCommand, StatusArg, TagOp, parse_create_tags, parse_update_tags};
use crate::db::{ApiAdminProject, ApiProjectDetail, ApiTag, TerminalCast};
use crate::handlers::{CreateProjectRequest, UpdateProjectRequest};

/// Run a projects subcommand
pub async fn run(client: ApiClient, command: ProjectsCommand, json: bool) -> Result<(), CliError> {
    match command {
        ProjectsCommand::List => list(client, json).await,
        ProjectsCommand::Get { reference } => get(client, &reference, json).await,
        ProjectsCommand::Create {
            name,
            short_desc,
            slug,
            status,
            hidden,
            private,
            github_repo,
            demo_url,
            tags,
            accent,
            project_type,
            terminal_cast,
            related,
        } => {
            let tag_slugs = tags.map(|s| parse_create_tags(&s)).unwrap_or_default();
            create(
                client,
                &name,
                &short_desc,
                slug,
                status,
                hidden,
                private,
                github_repo,
                demo_url,
                tag_slugs,
                accent,
                project_type,
                terminal_cast,
                related,
                json,
            )
            .await
        }
        ProjectsCommand::Update {
            reference,
            name,
            slug,
            short_desc,
            status,
            hidden,
            private,
            github_repo,
            demo_url,
            tags,
            accent,
            project_type,
            terminal_cast,
            related,
        } => {
            let tag_ops = match tags {
                Some(s) => parse_update_tags(&s).map_err(CliError::invalid)?,
                None => vec![],
            };
            update(
                client,
                &reference,
                name,
                slug,
                short_desc,
                status,
                hidden,
                private,
                github_repo,
                demo_url,
                tag_ops,
                accent,
                project_type,
                terminal_cast,
                related,
                json,
            )
            .await
        }
        ProjectsCommand::Delete { reference } => delete(client, &reference, json).await,
        ProjectsCommand::Sync { reference, all } => sync(client, reference, all, json).await,
        ProjectsCommand::Content(cmd) => super::content::run(client, cmd, json).await,
    }
}

/// List all projects
async fn list(client: ApiClient, json: bool) -> Result<(), CliError> {
    let projects: Vec<ApiAdminProject> =
        decode_json(check_response(client.get("/api/projects").await?).await?).await?;

    if json {
        output::print_json(&projects)?;
    } else {
        output::print_projects_table(&projects);
    }

    Ok(())
}

/// Get a project by slug or UUID
async fn get(client: ApiClient, reference: &str, json: bool) -> Result<(), CliError> {
    let project = resolve_project(&client, reference).await?;

    if json {
        output::print_json(&project)?;
    } else {
        output::print_project(&project.project);
    }

    Ok(())
}

/// Create a new project
#[allow(clippy::too_many_arguments)]
async fn create(
    client: ApiClient,
    name: &str,
    short_desc: &str,
    slug: Option<String>,
    status: StatusArg,
    hidden: bool,
    private: bool,
    github_repo: Option<String>,
    demo_url: Option<String>,
    tag_slugs: Vec<String>,
    accent: Option<String>,
    project_type: Option<String>,
    terminal_cast: Option<String>,
    related: Option<String>,
    json: bool,
) -> Result<(), CliError> {
    // Resolve tag slugs to IDs
    let tag_ids = resolve_tag_ids(&client, &tag_slugs).await?;
    let related_ids = resolve_related(&client, related).await?;

    let request = CreateProjectRequest {
        name: name.to_string(),
        slug,
        short_description: short_desc.to_string(),
        status: status.into(),
        hidden,
        github_repo: github_repo.filter(|s| !s.is_empty()),
        demo_url: demo_url.filter(|s| !s.is_empty()),
        tag_ids,
        detail_content: None,
        project_type: project_type.filter(|s| !s.is_empty()),
        private,
        terminal_cast: terminal_cast
            .filter(|s| !s.is_empty())
            .map(|p| read_cast(&p))
            .transpose()?,
        accent_color: accent.filter(|s| !s.is_empty()),
        related_ids,
    };

    let project: ApiAdminProject =
        decode_json(check_response(client.post("/api/projects", &request).await?).await?).await?;

    if json {
        output::print_json(&project)?;
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
    status: Option<StatusArg>,
    hidden: Option<bool>,
    private: Option<bool>,
    github_repo: Option<String>,
    demo_url: Option<String>,
    tag_ops: Vec<TagOp>,
    accent: Option<String>,
    project_type: Option<String>,
    terminal_cast: Option<String>,
    related: Option<String>,
    json: bool,
) -> Result<(), CliError> {
    // First fetch the current project
    let current = resolve_project(&client, reference).await?;
    // PUT is a full replace, so carry the detail body through unchanged (it has
    // its own `content` commands); the v2 fields below are preserved unless a
    // flag overrides them.
    let detail_content = current.detail_content.clone();
    let current_terminal_cast = current.terminal_cast.clone();
    let current_related_ids: Vec<String> = current.related.iter().map(|r| r.id.clone()).collect();
    let current = current.project;

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

    let request = UpdateProjectRequest {
        name: name.unwrap_or(current.project.name),
        slug,
        short_description: short_desc.unwrap_or(current.project.short_description),
        status: status.map_or(current.status, Into::into),
        hidden: hidden.unwrap_or(current.hidden),
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
        detail_content,
        project_type: match project_type {
            Some(s) if s.is_empty() => None,
            Some(s) => Some(s),
            None => current.project_type,
        },
        private: private.unwrap_or(current.private),
        terminal_cast: match terminal_cast {
            Some(s) if s.is_empty() => None,
            Some(p) => Some(read_cast(&p)?),
            None => current_terminal_cast,
        },
        accent_color: match accent {
            Some(s) if s.is_empty() => None,
            Some(s) => Some(s),
            None => current.accent_color,
        },
        related_ids: match related {
            Some(s) if s.trim().is_empty() => vec![],
            Some(s) => resolve_related(&client, Some(s)).await?,
            None => current_related_ids,
        },
    };

    let project: ApiAdminProject = decode_json(
        check_response(
            client
                .put(&format!("/api/projects/{}", current.project.id), &request)
                .await?,
        )
        .await?,
    )
    .await?;

    if json {
        output::print_json(&project)?;
    } else {
        output::success(&format!("Updated project: {}", project.project.name));
        output::print_project(&project);
    }

    Ok(())
}

/// Delete a project
async fn delete(client: ApiClient, reference: &str, json: bool) -> Result<(), CliError> {
    // First resolve to get the ID
    let project = resolve_project(&client, reference).await?.project;

    let deleted: ApiAdminProject = decode_json(
        check_response(
            client
                .delete(&format!("/api/projects/{}", project.project.id))
                .await?,
        )
        .await?,
    )
    .await?;

    if json {
        output::print_json(&deleted)?;
    } else {
        output::success(&format!("Deleted project: {}", deleted.project.name));
    }

    Ok(())
}

/// Trigger a GitHub activity sync for one project, or all projects with `--all`.
async fn sync(
    client: ApiClient,
    reference: Option<String>,
    all: bool,
    json: bool,
) -> Result<(), CliError> {
    if all {
        #[derive(serde::Serialize, serde::Deserialize)]
        struct Enqueued {
            enqueued: usize,
        }
        let result: Enqueued = decode_json(
            check_response(
                client
                    .post("/api/github/sync", &serde_json::json!({}))
                    .await?,
            )
            .await?,
        )
        .await?;
        if json {
            output::print_json(&result)?;
        } else {
            output::success(&format!("Enqueued {} project(s) for sync", result.enqueued));
        }
        return Ok(());
    }

    let Some(reference) = reference else {
        return Err(CliError::invalid("provide a project ref or --all"));
    };

    let project: ApiAdminProject = decode_json(
        check_response(
            client
                .post(
                    &format!("/api/projects/{reference}/sync"),
                    &serde_json::json!({}),
                )
                .await?,
        )
        .await?,
    )
    .await?;

    if json {
        output::print_json(&project)?;
    } else {
        output::success(&format!("Synced project: {}", project.project.name));
        output::print_project(&project);
    }
    Ok(())
}

/// Resolve a project reference (slug or UUID) to a full project
async fn resolve_project(
    client: &ApiClient,
    reference: &str,
) -> Result<ApiProjectDetail, CliError> {
    // The single-project endpoint accepts both UUID and slug, and returns the
    // full detail (including detail_content) so update can preserve it.
    decode_json(check_response(client.get(&format!("/api/projects/{reference}")).await?).await?)
        .await
}

/// Read a terminal-cast transcript from a JSON file.
fn read_cast(path: &str) -> Result<TerminalCast, CliError> {
    let raw = std::fs::read_to_string(path).map_err(|source| CliError::Io {
        path: path.into(),
        source,
    })?;
    serde_json::from_str(&raw).map_err(|source| CliError::Json {
        path: path.to_string(),
        source,
    })
}

/// Resolve a comma-separated list of related project refs (slug or UUID) to IDs,
/// preserving authored order. Empty/absent yields no related projects.
async fn resolve_related(
    client: &ApiClient,
    related: Option<String>,
) -> Result<Vec<String>, CliError> {
    let Some(list) = related else {
        return Ok(vec![]);
    };
    let mut ids = Vec::new();
    for reference in list.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        ids.push(resolve_project_id(client, reference).await?);
    }
    Ok(ids)
}

/// Resolve a single project reference (slug or UUID) to its UUID.
async fn resolve_project_id(client: &ApiClient, reference: &str) -> Result<String, CliError> {
    Ok(resolve_project(client, reference).await?.project.project.id)
}

/// Resolve tag slugs to IDs
async fn resolve_tag_ids(client: &ApiClient, slugs: &[String]) -> Result<Vec<String>, CliError> {
    let mut ids = Vec::new();
    for slug in slugs {
        ids.push(resolve_tag_id(client, slug).await?);
    }
    Ok(ids)
}

/// Resolve a single tag slug or ID to an ID
async fn resolve_tag_id(client: &ApiClient, slug_or_id: &str) -> Result<String, CliError> {
    // Try as UUID first
    if Uuid::parse_str(slug_or_id).is_ok() {
        return Ok(slug_or_id.to_string());
    }

    // Otherwise look up by slug
    #[derive(serde::Deserialize)]
    struct TagResponse {
        tag: ApiTag,
    }

    let tag_response: TagResponse =
        decode_json(check_response(client.get(&format!("/api/tags/{slug_or_id}")).await?).await?)
            .await?;
    Ok(tag_response.tag.id)
}
