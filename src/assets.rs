use axum::{
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use include_dir::{include_dir, Dir};

static CLIENT_ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/web/build/client");

pub async fn serve_embedded_asset(uri: Uri) -> Response {
    let path = uri.path();

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
        }
        None => {
            tracing::debug!(path, "Embedded asset not found");
            (StatusCode::NOT_FOUND, "Asset not found").into_response()
        }
    }
}
