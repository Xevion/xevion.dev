//! CLI authentication: a browser device-authorization flow.
//!
//! `login` starts a request against the target API, opens the approval page in a
//! browser, and waits on a server-sent-events stream for the admin to confirm.
//! On approval the server returns a long-lived bearer token, which is saved into
//! the named target in the config.

use futures::StreamExt;
use std::error::Error;
use time::OffsetDateTime;

use crate::cli::client::{ApiClient, check_response};
use crate::cli::config::{Config, DEFAULT_API_NAME};
use crate::cli::output;
use crate::cli_auth::{CliAuthStatus, StartRequest, StartResponse};

/// Default URL used when authorizing the conventional `local` target for the
/// first time without an explicit `--url`.
const LOCAL_DEFAULT_URL: &str = "http://localhost:10237";

/// Run the browser device-auth flow and persist the resulting token.
pub async fn login(
    mut config: Config,
    api_override: Option<&str>,
    url: Option<String>,
    label: Option<String>,
    no_browser: bool,
) -> Result<(), Box<dyn Error>> {
    let name = api_override
        .map(str::to_string)
        .or_else(|| config.default.clone())
        .unwrap_or_else(|| DEFAULT_API_NAME.to_string());

    let base_url = url
        .or_else(|| config.url_for(&name))
        .or_else(|| (name == DEFAULT_API_NAME).then(|| LOCAL_DEFAULT_URL.to_string()))
        .ok_or_else(|| {
            format!("No URL known for target '{name}'. Pass --url <url> to authorize it.")
        })?;
    let base_url = base_url.trim_end_matches('/').to_string();

    let label = label.or_else(|| gethostname::gethostname().into_string().ok());

    let http = reqwest::Client::new();

    // 1. Start the request.
    let start: StartResponse = check_response(
        http.post(format!("{base_url}/api/auth/device/start"))
            .json(&StartRequest {
                label: label.clone(),
            })
            .send()
            .await?,
    )
    .await?
    .json()
    .await?;

    // 2. Build the approval URL and present it.
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
            config.set_target(&name, base_url, Some(token));
            config.save()?;
            output::success(&format!(
                "Authorized as {} for target '{name}'",
                nu_ansi_term::Style::new().bold().paint(&username)
            ));
            output::info(&format!("Saved to {}", config.path().display()));
            Ok(())
        }
        CliAuthStatus::Denied => {
            output::error("Authorization was denied in the browser");
            Ok(())
        }
        CliAuthStatus::Pending => {
            output::error("Authorization request expired before approval");
            Ok(())
        }
    }
}

/// Read the SSE stream until a terminal event arrives or the request expires.
async fn wait_for_approval(
    http: &reqwest::Client,
    base_url: &str,
    start: &StartResponse,
) -> Result<CliAuthStatus, Box<dyn Error>> {
    let events_url = format!("{base_url}/api/auth/device/events/{}", start.request_id);
    // Fall back to the request TTL if the clock skew makes the diff negative.
    #[allow(clippy::duration_suboptimal_units)]
    let timeout = (start.expires_at - OffsetDateTime::now_utc())
        .try_into()
        .unwrap_or_else(|_| std::time::Duration::from_secs(600));

    output::info("Waiting for approval…");

    let result = tokio::time::timeout(timeout, async {
        let response = check_response(http.get(&events_url).send().await?).await?;
        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            buffer.push_str(&String::from_utf8_lossy(&chunk?));

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

                let event: CliAuthStatus = serde_json::from_str(&data)?;
                if !matches!(event, CliAuthStatus::Pending) {
                    return Ok::<_, Box<dyn Error>>(event);
                }
            }
        }

        Ok(CliAuthStatus::Pending)
    })
    .await;

    match result {
        Ok(event) => event,
        Err(_) => Ok(CliAuthStatus::Pending),
    }
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
) -> Result<(), Box<dyn Error>> {
    if client.is_authenticated() {
        let _ = client.post("/api/logout", &()).await;
    }
    config.clear_token(&name);
    config.save()?;
    output::success(&format!("Logged out of target '{name}'"));
    Ok(())
}

/// List configured API targets, marking the default and which have tokens.
pub fn targets(config: &Config, json: bool) {
    if json {
        let entries: Vec<_> = config
            .api
            .iter()
            .map(|(name, entry)| {
                serde_json::json!({
                    "name": name,
                    "url": entry.url,
                    "default": config.default.as_deref() == Some(name),
                    "authenticated": entry.token.is_some(),
                })
            })
            .collect();
        println!("{}", serde_json::json!({ "targets": entries }));
        return;
    }

    if config.api.is_empty() {
        output::info("No API targets configured. Run 'xevion api login --url <url>'.");
        return;
    }

    let dim = nu_ansi_term::Style::new().dimmed();
    let bold = nu_ansi_term::Style::new().bold();
    for (name, entry) in &config.api {
        let is_default = config.default.as_deref() == Some(name);
        let marker = if is_default { "*" } else { " " };
        let auth = if entry.token.is_some() {
            nu_ansi_term::Color::Green.paint("authorized")
        } else {
            dim.paint("no token")
        };
        println!(
            "{marker} {}  {}  {auth}",
            bold.paint(name),
            dim.paint(&entry.url),
        );
    }
}

/// Check the current session status against the server.
pub async fn session(client: ApiClient, name: &str, json: bool) -> Result<(), Box<dyn Error>> {
    let response = client.get("/api/session").await?;
    let response = check_response(response).await?;
    let session: SessionResponse = response.json().await?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "authenticated": session.authenticated,
                "username": session.username,
                "sessionType": session.session_type,
                "expiresAt": session.expires_at,
                "target": name,
                "apiUrl": client.api_url,
            })
        );
    } else if session.authenticated {
        output::print_session(&session.username, &client.api_url);
        let dim = nu_ansi_term::Style::new().dimmed();
        println!("  {} {name}", dim.paint("Target:"));
        println!("  {} {}", dim.paint("Type:"), session.session_type);
        println!("  {} {}", dim.paint("Expires:"), session.expires_at);
    } else {
        output::error("Session expired or invalid");
    }

    Ok(())
}
