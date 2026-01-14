use axum::{
    http::{StatusCode, Uri, header},
    response::{IntoResponse, Response},
};
use include_dir::{Dir, include_dir};

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

fn serve_asset_by_path(path: &str) -> Response {
    if let Some(response) = try_serve_embedded_asset(path) {
        response
    } else {
        tracing::debug!(path, "Embedded asset not found");
        (StatusCode::NOT_FOUND, "Asset not found").into_response()
    }
}

/// Get a static file from the embedded CLIENT_ASSETS.
///
/// Static files are served from web/static/ and embedded at compile time.
///
/// # Arguments
/// * `path` - Path to the file (e.g., "publickey.asc")
///
/// # Returns
/// * `Some(&[u8])` - File content if file exists
/// * `None` - If file not found
pub fn get_static_file(path: &str) -> Option<&'static [u8]> {
    CLIENT_ASSETS.get_file(path).map(|f| f.contents())
}

/// Get prerendered error page HTML for a given status code.
///
/// Error pages are prerendered by SvelteKit and embedded at compile time.
/// The list of available error codes is defined in web/src/lib/error-codes.ts.
///
/// # Arguments
/// * `status_code` - HTTP status code (e.g., 404, 500)
///
/// # Returns
/// * `Some(&[u8])` - HTML content if error page exists
/// * `None` - If no prerendered page exists for this code
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
/// Prerendered content is built by SvelteKit at compile time and embedded.
/// This serves any file from the prerendered directory with appropriate MIME types.
///
/// Path resolution order:
/// 1. Exact file match (e.g., `/pgp/__data.json` → `pgp/__data.json`)
/// 2. HTML file for extensionless paths (e.g., `/pgp` → `pgp.html`)
/// 3. Index file for directory paths (e.g., `/about/` → `about/index.html`)
///
/// # Arguments
/// * `path` - Request path (e.g., "/pgp", "/pgp/__data.json")
///
/// # Returns
/// * `Some(Response)` - Response with appropriate content-type if file exists
/// * `None` - If no prerendered content exists for this path
pub fn try_serve_prerendered_page(path: &str) -> Option<Response> {
    let path = path.strip_prefix('/').unwrap_or(path);

    // Try exact file match first (handles __data.json, etc.)
    if let Some(file) = PRERENDERED_PAGES.get_file(path) {
        return Some(serve_prerendered_file(path, file.contents()));
    }

    let path = path.strip_suffix('/').unwrap_or(path);

    // Try as HTML file: "pgp" -> "pgp.html"
    let html_path = format!("{}.html", path);
    if let Some(file) = PRERENDERED_PAGES.get_file(&html_path) {
        return Some(serve_prerendered_file(&html_path, file.contents()));
    }

    // Try index pattern: "path" -> "path/index.html"
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
