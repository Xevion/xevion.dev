use axum::{
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};

use crate::assets;

/// Check if a path represents a static asset
pub fn is_static_asset(path: &str) -> bool {
    path.starts_with("/node_modules/")
        || path.starts_with("/@") // Vite internals like /@vite/client, /@fs/, /@id/
        || path.starts_with("/.svelte-kit/")
        || path.starts_with("/.well-known/")
        || path.ends_with(".woff2")
        || path.ends_with(".woff")
        || path.ends_with(".ttf")
        || path.ends_with(".ico")
        || path.ends_with(".png")
        || path.ends_with(".jpg")
        || path.ends_with(".svg")
        || path.ends_with(".webp")
        || path.ends_with(".css")
        || path.ends_with(".js")
        || path.ends_with(".map")
}

/// Check if a path represents a page route (not an asset)
pub fn is_page_route(path: &str) -> bool {
    !path.starts_with("/node_modules/")
        && !path.starts_with("/@")
        && !path.starts_with("/.svelte-kit/")
        && !path.contains('.')
}

/// Check if the request accepts HTML responses
pub fn accepts_html(headers: &HeaderMap) -> bool {
    if let Some(accept) = headers.get(header::ACCEPT) {
        if let Ok(accept_str) = accept.to_str() {
            return accept_str.contains("text/html") || accept_str.contains("*/*");
        }
    }
    // Default to true for requests without Accept header (browsers typically send it)
    true
}

/// Determines if request prefers raw content (CLI tools) over HTML
pub fn prefers_raw_content(headers: &HeaderMap) -> bool {
    // Check User-Agent for known CLI tools first (most reliable)
    if let Some(ua) = headers.get(header::USER_AGENT) {
        if let Ok(ua_str) = ua.to_str() {
            let ua_lower = ua_str.to_lowercase();
            if ua_lower.starts_with("curl/")
                || ua_lower.starts_with("wget/")
                || ua_lower.starts_with("httpie/")
                || ua_lower.contains("curlie")
            {
                return true;
            }
        }
    }

    // Check Accept header - if it explicitly prefers text/html, serve HTML
    if let Some(accept) = headers.get(header::ACCEPT) {
        if let Ok(accept_str) = accept.to_str() {
            // If text/html appears before */* in the list, they prefer HTML
            if let Some(html_pos) = accept_str.find("text/html") {
                if let Some(wildcard_pos) = accept_str.find("*/*") {
                    return html_pos > wildcard_pos;
                }
                // Has text/html but no */* → prefers HTML
                return false;
            }
            // Has */* but no text/html → probably a CLI tool
            if accept_str.contains("*/*") && !accept_str.contains("text/html") {
                return true;
            }
        }
    }

    // No Accept header → assume browser (safer default)
    false
}

/// Serve a prerendered error page for the given status code
pub fn serve_error_page(status: StatusCode) -> Response {
    let status_code = status.as_u16();

    if let Some(html) = assets::get_error_page(status_code) {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/html; charset=utf-8"),
        );
        headers.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-cache, no-store, must-revalidate"),
        );

        (status, headers, html).into_response()
    } else {
        // Fallback for undefined error codes (500 generic page)
        tracing::warn!(
            status_code,
            "No prerendered error page found for status code - using fallback"
        );

        if let Some(fallback_html) = assets::get_error_page(500) {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            );
            headers.insert(
                header::CACHE_CONTROL,
                HeaderValue::from_static("no-cache, no-store, must-revalidate"),
            );

            (status, headers, fallback_html).into_response()
        } else {
            // Last resort: plaintext (should never happen if 500.html exists)
            (status, format!("Error {}", status_code)).into_response()
        }
    }
}

/// Validate hex color format (6 characters, no hash, no alpha)
pub fn validate_hex_color(color: &str) -> bool {
    color.len() == 6 && color.chars().all(|c| c.is_ascii_hexdigit())
}
