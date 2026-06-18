//! SEO discovery routes: `/robots.txt` and `/sitemap.xml`.
//!
//! Both are served directly from Rust (no Bun roundtrip) so they stay available
//! even when SSR is down, and they build absolute URLs from the request's
//! allowlist-validated public host (see [`crate::host`] / XEV-986) — so each
//! public domain (`xevion.dev`, `walters.to`, …) gets a sitemap rooted at its own
//! origin. The project URL set is cached for 15 minutes to keep crawler traffic
//! off the database; the per-host origin is stamped on each render, so a single
//! cache entry serves every domain.

use std::fmt::Write as _;
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use time::OffsetDateTime;
use tokio::sync::RwLock;

use crate::{db::SitemapEntry, state::AppState};

/// How long a fetched project URL set is reused before re-querying.
const SITEMAP_TTL: Duration = Duration::from_mins(15);

struct CachedEntries {
    entries: Vec<SitemapEntry>,
    fetched_at: Instant,
}

static SITEMAP_CACHE: LazyLock<RwLock<Option<CachedEntries>>> = LazyLock::new(|| RwLock::new(None));

/// Absolute public origin (e.g. `https://xevion.dev`) for this request.
fn request_origin(state: &AppState, headers: &HeaderMap) -> String {
    format!(
        "{}://{}",
        state.host_config.scheme(),
        state.host_config.resolve(headers)
    )
}

/// `GET /robots.txt` — allow-all with admin/api/internal carve-outs and an
/// absolute sitemap pointer rooted at the request's public origin.
pub async fn robots_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let origin = request_origin(&state, &headers);
    let body = format!(
        "User-agent: *\n\
         Allow: /\n\
         Disallow: /admin/\n\
         Disallow: /api/\n\
         Disallow: /internal/\n\
         \n\
         Sitemap: {origin}/sitemap.xml\n"
    );

    (
        [
            (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=86400"),
        ],
        body,
    )
}

/// `GET /sitemap.xml` — homepage, PGP page, and every public project URL, with
/// `<lastmod>` for project pages.
pub async fn sitemap_handler(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let entries = match cached_entries(&state).await {
        Ok(entries) => entries,
        Err(e) => {
            tracing::error!(error = %e, "Failed to load sitemap entries");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let origin = request_origin(&state, &headers);
    let xml = render_sitemap(&origin, &entries);

    (
        [
            (header::CONTENT_TYPE, "application/xml; charset=utf-8"),
            (
                header::CACHE_CONTROL,
                "public, max-age=900, stale-while-revalidate=1800",
            ),
        ],
        xml,
    )
        .into_response()
}

/// Return the cached project URL set, refreshing from the database when the entry
/// is missing or older than [`SITEMAP_TTL`].
async fn cached_entries(state: &AppState) -> Result<Vec<SitemapEntry>, sqlx::Error> {
    {
        let cache = SITEMAP_CACHE.read().await;
        if let Some(cached) = cache.as_ref()
            && cached.fetched_at.elapsed() < SITEMAP_TTL
        {
            return Ok(cached.entries.clone());
        }
    }

    let entries = crate::db::list_sitemap_entries(&state.pool).await?;
    {
        let mut cache = SITEMAP_CACHE.write().await;
        *cache = Some(CachedEntries {
            entries: entries.clone(),
            fetched_at: Instant::now(),
        });
    }
    Ok(entries)
}

/// Build the `<urlset>` document with absolute `<loc>`s under `origin`.
///
/// Project slugs are `[a-z0-9-]` (see [`crate::db::slugify`]) and the static paths
/// are literals, so no value here carries XML-special characters to escape.
fn render_sitemap(origin: &str, entries: &[SitemapEntry]) -> String {
    let mut xml = String::with_capacity(512 + entries.len() * 160);
    xml.push_str(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );

    write_url(&mut xml, origin, "/", None);
    write_url(&mut xml, origin, "/pgp", None);

    for entry in entries {
        let path = format!("/projects/{}", entry.slug);
        write_url(&mut xml, origin, &path, Some(entry.lastmod));
    }

    xml.push_str("</urlset>\n");
    xml
}

fn write_url(xml: &mut String, origin: &str, path: &str, lastmod: Option<OffsetDateTime>) {
    xml.push_str("  <url>\n");
    let _ = writeln!(xml, "    <loc>{origin}{path}</loc>");
    if let Some(ts) = lastmod {
        let date = ts.date();
        let _ = writeln!(
            xml,
            "    <lastmod>{:04}-{:02}-{:02}</lastmod>",
            date.year(),
            u8::from(date.month()),
            date.day()
        );
    }
    xml.push_str("  </url>\n");
}
