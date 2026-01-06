use crate::auth::{Session, SessionManager};
use axum::{
    Json,
    body::Body,
    extract::{Request, State},
    http::{StatusCode, Uri},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;
use serde_json::json;
use std::sync::Arc;

const SESSION_COOKIE_NAME: &str = "admin_session";

pub async fn require_admin_auth(
    State(session_mgr): State<Arc<SessionManager>>,
    jar: CookieJar,
    uri: Uri,
    mut req: Request,
    next: Next,
) -> Result<Response, Response> {
    let session_cookie = jar.get(SESSION_COOKIE_NAME);

    let session_id = session_cookie.and_then(|cookie| ulid::Ulid::from_string(cookie.value()).ok());

    let session = session_id.and_then(|id| session_mgr.validate_session(id));

    match session {
        Some(session) => {
            req.extensions_mut().insert(session);
            Ok(next.run(req).await)
        }
        None => {
            let next_param = urlencoding::encode(uri.path());
            let redirect_url = format!("/admin/login?next={}", next_param);
            Err(Redirect::to(&redirect_url).into_response())
        }
    }
}

pub async fn require_api_auth(
    State(session_mgr): State<Arc<SessionManager>>,
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<Response, Response> {
    let session_cookie = jar.get(SESSION_COOKIE_NAME);

    let session_id = session_cookie.and_then(|cookie| ulid::Ulid::from_string(cookie.value()).ok());

    let session = session_id.and_then(|id| session_mgr.validate_session(id));

    match session {
        Some(session) => {
            req.extensions_mut().insert(session);
            Ok(next.run(req).await)
        }
        None => {
            let error_response = (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "Unauthorized",
                    "message": "Authentication required"
                })),
            );
            Err(error_response.into_response())
        }
    }
}

pub fn extract_session(req: &Request<Body>) -> Option<Session> {
    req.extensions().get::<Session>().cloned()
}
