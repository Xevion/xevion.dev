use axum::{
    http::{HeaderMap, StatusCode, Uri, header},
    response::{IntoResponse, Response},
};
use include_dir::{Dir, include_dir};

use crate::encoding;

static CLIENT_ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/web/build/client");
static ERROR_PAGES: Dir = include_dir!("$CARGO_MANIFEST_DIR/web/build/prerendered/errors");
static PRERENDERED_PAGES: Dir = include_dir!("$CARGO_MANIFEST_DIR/web/build/prerendered");
static ENV_JS: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/web/build/env.js"));

pub async fn serve_embedded_asset(uri: Uri) -> Response {
    let path = uri.path();
    serve_asset_by_path(path)
}

/// Serve an embedded asset by path, or return None if not found
pub fn try_serve_embedded_asset(path: &str) -> Option<Response> {
    let asset_path = path.strip_prefix('/').unwrap_or(path);

    CLIENT_ASSETS.get_file(asset_path).map(|file| {
        let mime_type = mime_guess::from_path(asset_path)
            .first_or_octet_stream()
            .as_ref()
            .to_string();

        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            mime_type
                .parse()
                .unwrap_or_else(|_| header::HeaderValue::from_static("application/octet-stream")),
        );

        if path.contains("/immutable/") {
            headers.insert(
                header::CACHE_CONTROL,
                header::HeaderValue::from_static("public, max-age=31536000, immutable"),
            );
        } else {
            headers.insert(
                header::CACHE_CONTROL,
                header::HeaderValue::from_static("public, max-age=3600"),
            );
        }

        (StatusCode::OK, headers, file.contents()).into_response()
    })
}

/// Serve an embedded asset with content encoding negotiation.
///
/// Attempts to serve pre-compressed variants (.br, .gz, .zst) based on
/// Accept-Encoding. Falls back to uncompressed if no suitable variant is found.
/// Pre-compressed assets are generated at build time by scripts/compress-assets.ts.
pub fn try_serve_embedded_asset_with_encoding(path: &str, headers: &HeaderMap) -> Option<Response> {
    let asset_path = path.strip_prefix('/').unwrap_or(path);

    // Parse accepted encodings in priority order
    let accepted_encodings = encoding::parse_accepted_encodings(headers);

    // Try each encoding in order of client preference
    for encoding in accepted_encodings {
        // Skip identity - we'll use it as final fallback
        if encoding == encoding::ContentEncoding::Identity {
            continue;
        }

        // Build path to pre-compressed variant
        let compressed_path = format!("{}{}", asset_path, encoding.extension());

        // Check if pre-compressed variant exists
        if let Some(file) = CLIENT_ASSETS.get_file(&compressed_path) {
            // Get MIME type from ORIGINAL path (not .br/.gz/.zst extension)
            let mime_type = mime_guess::from_path(asset_path)
                .first_or_octet_stream()
                .as_ref()
                .to_string();

            let mut response_headers = axum::http::HeaderMap::new();
            response_headers.insert(
                header::CONTENT_TYPE,
                mime_type.parse().unwrap_or_else(|_| {
                    header::HeaderValue::from_static("application/octet-stream")
                }),
            );

            // Set Content-Encoding header
            if let Some(encoding_value) = encoding.header_value() {
                response_headers.insert(header::CONTENT_ENCODING, encoding_value);
            }

            // Set cache headers (same as uncompressed)
            if path.contains("/immutable/") {
                response_headers.insert(
                    header::CACHE_CONTROL,
                    header::HeaderValue::from_static("public, max-age=31536000, immutable"),
                );
            } else {
                response_headers.insert(
                    header::CACHE_CONTROL,
                    header::HeaderValue::from_static("public, max-age=3600"),
                );
            }

            return Some((StatusCode::OK, response_headers, file.contents()).into_response());
        }
    }

    try_serve_embedded_asset(path)
}

fn serve_asset_by_path(path: &str) -> Response {
    if let Some(response) = try_serve_embedded_asset(path) {
        response
    } else {
        tracing::debug!(path, "Embedded asset not found");
        (StatusCode::NOT_FOUND, "Asset not found").into_response()
    }
}

/// Get a static file from the embedded CLIENT_ASSETS.
pub fn get_static_file(path: &str) -> Option<&'static [u8]> {
    CLIENT_ASSETS.get_file(path).map(|f| f.contents())
}

/// Get prerendered error page HTML for a given status code.
/// Available codes defined in web/src/lib/error-codes.ts.
pub fn get_error_page(status_code: u16) -> Option<&'static [u8]> {
    let filename = format!("{}.html", status_code);
    ERROR_PAGES.get_file(&filename).map(|f| f.contents())
}

/// Get the embedded SvelteKit env.js file for dynamic public environment variables.
///
/// SvelteKit generates this file when using `$env/dynamic/public` imports.
/// It must be served at `/_app/env.js` for prerendered pages to hydrate correctly.
pub fn get_env_js() -> &'static [u8] {
    ENV_JS
}

/// Serve prerendered content by path, if it exists.
///
/// Resolution: exact file → `{path}.html` → `{path}/index.html`
pub fn try_serve_prerendered_page(path: &str) -> Option<Response> {
    let path = path.strip_prefix('/').unwrap_or(path);

    // Try exact file match first (handles __data.json, etc.)
    if let Some(file) = PRERENDERED_PAGES.get_file(path) {
        return Some(serve_prerendered_file(path, file.contents()));
    }

    let path = path.strip_suffix('/').unwrap_or(path);

    let html_path = format!("{}.html", path);
    if let Some(file) = PRERENDERED_PAGES.get_file(&html_path) {
        return Some(serve_prerendered_file(&html_path, file.contents()));
    }

    let index_path = if path.is_empty() {
        "index.html".to_string()
    } else {
        format!("{}/index.html", path)
    };
    if let Some(file) = PRERENDERED_PAGES.get_file(&index_path) {
        return Some(serve_prerendered_file(&index_path, file.contents()));
    }

    None
}

fn serve_prerendered_file(path: &str, content: &'static [u8]) -> Response {
    let mime_type = mime_guess::from_path(path)
        .first_or_octet_stream()
        .as_ref()
        .to_string();

    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        mime_type
            .parse()
            .unwrap_or_else(|_| header::HeaderValue::from_static("application/octet-stream")),
    );
    headers.insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("public, max-age=3600"),
    );

    (StatusCode::OK, headers, content).into_response()
}
