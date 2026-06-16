//! CLI authentication: a browser device-authorization flow, plus target and
//! config management.
//!
//! `login` starts a request against the target API, opens the approval page in a
//! browser, and waits on a server-sent-events stream for the admin to confirm.
//! On approval the server returns a long-lived bearer token, which is saved into
//! the named target in the config.

use futures::StreamExt;
use time::OffsetDateTime;

use crate::cli::client::{ApiClient, check_response, json as decode_json};
use crate::cli::config::Config;
use crate::cli::error::CliError;
use crate::cli::{TargetsCommand, output};
use crate::cli_auth::{CliAuthStatus, StartRequest, StartResponse};

/// Run the browser device-auth flow and persist the resulting token.
pub async fn login(
    mut config: Config,
    api_override: Option<&str>,
    url: Option<String>,
    label: Option<String>,
    no_browser: bool,
    json: bool,
) -> Result<(), CliError> {
    // Target name: explicit --api, else the configured default. No magic names —
    // a first-time user must say which target they are authorizing.
    let name = api_override
        .map(str::to_string)
        .or_else(|| config.default.clone())
        .ok_or_else(|| {
            CliError::invalid(
                "no target selected: pass --api <name> (and --url <url> for a new target)",
            )
        })?;

    // URL: explicit --url, else the target's stored URL. No localhost fallback.
    let base_url = url.or_else(|| config.url_for(&name)).ok_or_else(|| {
        CliError::invalid(format!(
            "no URL known for target '{name}': pass --url <url> to authorize it"
        ))
    })?;
    let base_url = base_url.trim_end_matches('/').to_string();

    let label = label.or_else(|| gethostname::gethostname().into_string().ok());

    let http = reqwest::Client::new();

    // 1. Start the request.
    let start: StartResponse = decode_json(
        check_response(
            http.post(format!("{base_url}/api/auth/device/start"))
                .json(&StartRequest {
                    label: label.clone(),
                })
                .send()
                .await
                .map_err(|e| CliError::from_send(format!("{base_url}/api/auth/device/start"), e))?,
        )
        .await?,
    )
    .await?;

    // 2. Build the approval URL and present it (progress → stderr).
    let approval_url = format!(
        "{base_url}{path}?request={id}&code={code}",
        path = start.verification_path,
        id = start.request_id,
        code = start.user_code,
    );

    output::info(&format!(
        "Confirm this code in your browser: {}",
        nu_ansi_term::Style::new().bold().paint(&start.user_code)
    ));
    output::info(&format!("Approval URL: {approval_url}"));

    if no_browser {
        output::info("Open the URL above to approve.");
    } else if let Err(e) = open::that(&approval_url) {
        output::error(&format!(
            "Couldn't open a browser ({e}); open the URL above manually."
        ));
    }

    // 3. Wait on the SSE stream for approval/denial.
    let event = wait_for_approval(&http, &base_url, &start).await?;

    match event {
        CliAuthStatus::Approved { token, username } => {
            config.set_target(&name, base_url.clone(), Some(token));
            config.save()?;
            if json {
                output::print_json(&serde_json::json!({
                    "target": name,
                    "url": base_url,
                    "username": username,
                    "authenticated": true,
                }))?;
            } else {
                output::success(&format!(
                    "Authorized as {} for target '{name}'",
                    nu_ansi_term::Style::new().bold().paint(&username)
                ));
                output::info(&format!("Saved to {}", config.path().display()));
            }
            Ok(())
        }
        CliAuthStatus::Denied => Err(CliError::Approval {
            reason: "was denied in the browser".to_string(),
        }),
        CliAuthStatus::Pending => Err(CliError::Approval {
            reason: "request expired before approval".to_string(),
        }),
    }
}

/// Read the SSE stream until a terminal event arrives or the request expires.
async fn wait_for_approval(
    http: &reqwest::Client,
    base_url: &str,
    start: &StartResponse,
) -> Result<CliAuthStatus, CliError> {
    let events_url = format!("{base_url}/api/auth/device/events/{}", start.request_id);
    // Fall back to the request TTL if the clock skew makes the diff negative.
    #[allow(clippy::duration_suboptimal_units)]
    let timeout = (start.expires_at - OffsetDateTime::now_utc())
        .try_into()
        .unwrap_or_else(|_| std::time::Duration::from_secs(600));

    output::info("Waiting for approval…");

    let result = tokio::time::timeout(timeout, async {
        let response = check_response(
            http.get(&events_url)
                .send()
                .await
                .map_err(|e| CliError::from_send(events_url.clone(), e))?,
        )
        .await?;
        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| CliError::from_send(events_url.clone(), e))?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            // SSE frames are separated by a blank line.
            while let Some(idx) = buffer.find("\n\n") {
                let frame = buffer[..idx].to_string();
                buffer.drain(..idx + 2);

                let data: String = frame
                    .lines()
                    .filter_map(|line| line.strip_prefix("data:"))
                    .map(str::trim_start)
                    .collect::<Vec<_>>()
                    .join("\n");

                if data.is_empty() {
                    continue;
                }

                let event: CliAuthStatus =
                    serde_json::from_str(&data).map_err(CliError::invalid)?;
                if !matches!(event, CliAuthStatus::Pending) {
                    return Ok::<_, CliError>(event);
                }
            }
        }

        Ok(CliAuthStatus::Pending)
    })
    .await;

    // A timeout reads as "still pending"; login turns that into an Approval error.
    result.unwrap_or(Ok(CliAuthStatus::Pending))
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionResponse {
    authenticated: bool,
    username: String,
    session_type: String,
    expires_at: String,
}

/// Revoke the saved token server-side and clear it from the config.
pub async fn logout(
    mut config: Config,
    name: String,
    client: ApiClient,
    json: bool,
) -> Result<(), CliError> {
    // Best-effort server-side revoke; clear locally regardless.
    let _ = client.post("/api/logout", &()).await;
    config.clear_token(&name);
    config.save()?;
    if json {
        output::print_json(&serde_json::json!({ "target": name, "authenticated": false }))?;
    } else {
        output::success(&format!("Logged out of target '{name}'"));
    }
    Ok(())
}

/// Dispatch `targets` and its management subcommands.
pub fn targets(
    mut config: Config,
    command: Option<TargetsCommand>,
    json: bool,
) -> Result<(), CliError> {
    match command {
        None => {
            list_targets(&config, json)?;
            Ok(())
        }
        Some(TargetsCommand::Use { name }) => {
            config.set_default(&name)?;
            config.save()?;
            confirm_target("default", &name, &config, json)
        }
        Some(TargetsCommand::Add { name, url }) => {
            config.set_target(&name, url, None);
            config.save()?;
            confirm_target("added", &name, &config, json)
        }
        Some(TargetsCommand::Rm { name }) => {
            let entry = config.remove_target(&name)?;
            config.save()?;
            if !json && entry.token.is_some() {
                output::info(
                    "Removed an authorized target; its server-side token was not revoked (use `logout` first to revoke).",
                );
            }
            confirm_target("removed", &name, &config, json)
        }
        Some(TargetsCommand::Rename { from, to }) => {
            config.rename_target(&from, &to)?;
            config.save()?;
            confirm_target("renamed", &to, &config, json)
        }
    }
}

/// Report a target mutation. `action` is a stable machine code (`default`,
/// `added`, `removed`, `renamed`); human mode renders a readable confirmation on
/// stderr, JSON mode prints the action object on stdout.
fn confirm_target(action: &str, name: &str, config: &Config, json: bool) -> Result<(), CliError> {
    if json {
        return output::print_json(&serde_json::json!({
            "action": action,
            "target": name,
            "default": config.default.as_deref() == Some(name),
        }));
    }
    let human = match action {
        "default" => format!("Target '{name}' is now the default"),
        "added" => format!("Target '{name}' added"),
        "removed" => format!("Target '{name}' removed"),
        "renamed" => format!("Renamed target to '{name}'"),
        _ => format!("Target '{name}' {action}"),
    };
    output::success(&human);
    Ok(())
}

/// List configured API targets, marking the default and which have tokens.
fn list_targets(config: &Config, json: bool) -> Result<(), CliError> {
    if json {
        let entries: Vec<_> = config
            .names()
            .map(|name| {
                serde_json::json!({
                    "name": name,
                    "url": config.url_for(name),
                    "default": config.default.as_deref() == Some(name),
                    "authorized": config.is_authorized(name),
                })
            })
            .collect();
        return output::print_json(&serde_json::json!({
            "targets": entries,
            "path": config.path().display().to_string(),
        }));
    }

    if config.names().next().is_none() {
        output::info(
            "No API targets configured. Add one: xevion api targets add <name> --url <url>",
        );
        return Ok(());
    }

    let dim = nu_ansi_term::Style::new().dimmed();
    let bold = nu_ansi_term::Style::new().bold();
    for name in config.names() {
        let is_default = config.default.as_deref() == Some(name);
        let marker = if is_default { "*" } else { " " };
        let auth = if config.is_authorized(name) {
            nu_ansi_term::Color::Green.paint("authorized")
        } else {
            dim.paint("no token")
        };
        println!(
            "{marker} {}  {}  {auth}",
            bold.paint(name),
            dim.paint(config.url_for(name).unwrap_or_default()),
        );
    }
    output::info(&format!("config: {}", config.path().display()));
    Ok(())
}

/// Print the resolved config file path. Human mode prints the bare path to stdout
/// so it is scriptable (`cd $(xevion api config path | xargs dirname)`).
pub fn config_path(config: &Config, json: bool) {
    if json {
        println!(
            "{}",
            serde_json::json!({ "path": config.path().display().to_string() })
        );
    } else {
        println!("{}", config.path().display());
    }
}

/// Check the current session status against the server.
pub async fn session(client: ApiClient, name: &str, json: bool) -> Result<(), CliError> {
    let response = check_response(client.get("/api/session").await?).await?;
    let session: SessionResponse = decode_json(response).await?;

    if json {
        output::print_json(&serde_json::json!({
            "authenticated": session.authenticated,
            "username": session.username,
            "sessionType": session.session_type,
            "expiresAt": session.expires_at,
            "target": name,
            "apiUrl": client.api_url,
        }))?;
    } else if session.authenticated {
        output::print_session(&session.username, &client.api_url);
        let dim = nu_ansi_term::Style::new().dimmed();
        eprintln!("  {} {name}", dim.paint("Target:"));
        eprintln!("  {} {}", dim.paint("Type:"), session.session_type);
        eprintln!("  {} {}", dim.paint("Expires:"), session.expires_at);
    } else {
        output::error("Session expired or invalid");
    }

    Ok(())
}
