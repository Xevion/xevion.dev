use axum::{
    http::{StatusCode, Uri, header},
    response::{IntoResponse, Response},
};
use include_dir::{Dir, include_dir};

static CLIENT_ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/web/build/client");
static ERROR_PAGES: Dir = include_dir!("$CARGO_MANIFEST_DIR/web/build/prerendered/errors");
static PRERENDERED_PAGES: Dir = include_dir!("$CARGO_MANIFEST_DIR/web/build/prerendered");

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

/// Serve a prerendered page by path, if it exists.
///
/// Prerendered pages are built by SvelteKit at compile time and embedded.
/// This handles various path patterns:
/// - `/path` → looks for `path.html`
/// - `/path/` → looks for `path.html` or `path/index.html`
///
/// # Arguments
/// * `path` - Request path (e.g., "/pgp", "/about/")
///
/// # Returns
/// * `Some(Response)` - HTML response if prerendered page exists
/// * `None` - If no prerendered page exists for this path
pub fn try_serve_prerendered_page(path: &str) -> Option<Response> {
    let path = path.strip_prefix('/').unwrap_or(path);
    let path = path.strip_suffix('/').unwrap_or(path);

    // Try direct HTML file first: "pgp" -> "pgp.html"
    let html_filename = format!("{}.html", path);
    if let Some(file) = PRERENDERED_PAGES.get_file(&html_filename) {
        return Some(serve_html_response(file.contents()));
    }

    // Try index.html pattern: "path" -> "path/index.html"
    let index_filename = format!("{}/index.html", path);
    if let Some(file) = PRERENDERED_PAGES.get_file(&index_filename) {
        return Some(serve_html_response(file.contents()));
    }

    // Try root index: "" -> "index.html"
    if path.is_empty() {
        if let Some(file) = PRERENDERED_PAGES.get_file("index.html") {
            return Some(serve_html_response(file.contents()));
        }
    }

    None
}

fn serve_html_response(content: &'static [u8]) -> Response {
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("text/html; charset=utf-8"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("public, max-age=3600"),
    );

    (StatusCode::OK, headers, content).into_response()
}
