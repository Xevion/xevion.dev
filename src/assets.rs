use axum::{
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use include_dir::{include_dir, Dir};

/// Embedded client assets from the SvelteKit build
/// These are the static JS/CSS bundles that get served to browsers
static CLIENT_ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/web/build/client");

/// Serves embedded client assets from the /_app path
/// Returns 404 if the asset doesn't exist
pub async fn serve_embedded_asset(uri: Uri) -> Response {
    let path = uri.path();

    // Strip leading slash for lookup
    let asset_path = path.strip_prefix('/').unwrap_or(path);

    match CLIENT_ASSETS.get_file(asset_path) {
        Some(file) => {
            let mime_type = mime_guess::from_path(asset_path)
                .first_or_octet_stream()
                .as_ref()
                .to_string();

            let mut headers = axum::http::HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                mime_type.parse().unwrap_or_else(|_| {
                    header::HeaderValue::from_static("application/octet-stream")
                }),
            );

            // Immutable assets can be cached forever (they're content-hashed)
            if path.contains("/immutable/") {
                headers.insert(
                    header::CACHE_CONTROL,
                    header::HeaderValue::from_static("public, max-age=31536000, immutable"),
                );
            } else {
                // Version file and other assets get short cache
                headers.insert(
                    header::CACHE_CONTROL,
                    header::HeaderValue::from_static("public, max-age=3600"),
                );
            }

            (StatusCode::OK, headers, file.contents()).into_response()
        }
        None => {
            tracing::debug!(path, "Embedded asset not found");
            (StatusCode::NOT_FOUND, "Asset not found").into_response()
        }
    }
}
