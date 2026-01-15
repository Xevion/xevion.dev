use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

use crate::cli::client::{ApiClient, Session, check_response};
use crate::cli::output;

#[derive(Serialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginResponse {
    success: bool,
    username: String,
}

#[derive(Deserialize)]
struct SessionResponse {
    authenticated: bool,
    username: String,
}

/// Login and save session
pub async fn login(
    mut client: ApiClient,
    username: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = LoginRequest {
        username: username.to_string(),
        password: password.to_string(),
    };

    let response = client.post("/api/login", &request).await?;

    // Extract session cookie from response headers
    let session_token = response
        .headers()
        .get_all("set-cookie")
        .iter()
        .find_map(|v| {
            let s = v.to_str().ok()?;
            if s.starts_with("admin_session=") {
                // Extract just the token value
                let token = s.strip_prefix("admin_session=")?.split(';').next()?;
                Some(token.to_string())
            } else {
                None
            }
        });

    let response = check_response(response).await?;
    let login_response: LoginResponse = response.json().await?;

    if !login_response.success {
        output::error("Login failed");
        return Ok(());
    }

    let session_token = session_token.ok_or("No session cookie received")?;

    let session = Session {
        api_url: client.api_url.clone(),
        session_token,
        username: login_response.username.clone(),
        created_at: OffsetDateTime::now_utc(),
        expires_at: OffsetDateTime::now_utc() + Duration::days(7),
    };

    client.save_session(session)?;

    output::success(&format!("Logged in as {}", login_response.username));
    Ok(())
}

/// Clear saved session
pub async fn logout(mut client: ApiClient) -> Result<(), Box<dyn std::error::Error>> {
    // Try to call logout endpoint if we have a session
    if client.is_authenticated() {
        let _ = client.post("/api/logout", &()).await;
    }

    client.clear_session()?;
    output::success("Logged out");
    Ok(())
}

/// Check current session status
pub async fn session(client: ApiClient, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    // First check local session
    if let Some(session) = client.session() {
        // Verify with server
        let response = client.get("/api/session").await?;
        let response = check_response(response).await?;
        let session_response: SessionResponse = response.json().await?;

        if json {
            println!(
                "{}",
                serde_json::json!({
                    "authenticated": session_response.authenticated,
                    "username": session_response.username,
                    "api_url": session.api_url,
                    "expires_at": session.expires_at.format(&time::format_description::well_known::Rfc3339).unwrap(),
                })
            );
        } else if session_response.authenticated {
            output::print_session(&session_response.username, &session.api_url);
        } else {
            output::error("Session expired or invalid");
        }
    } else if json {
        println!(
            "{}",
            serde_json::json!({
                "authenticated": false,
            })
        );
    } else {
        output::info("Not logged in");
    }

    Ok(())
}
