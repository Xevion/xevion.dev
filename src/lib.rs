//! Shared library for the `xevion` CLI and `xevion-server` binaries.
//!
//! Both binaries link this single crate; the server uses the HTTP/DB/asset
//! modules while the CLI uses [`cli::api`] (plus the shared DTO types in
//! [`db`], [`handlers`], [`pm`], and [`cli_auth`]). See `src/bin/` for the two
//! entry points.

// Shared between the `xevion` CLI and `xevion-server`: the CLI dispatch, the
// DTO types (db/handlers/pm/cli_auth), and the error types in `state`. These
// compile without the `server` feature so the client builds with no frontend.
pub mod cli;
pub mod cli_auth;
pub mod db;
pub mod handlers;
pub mod markdown;
pub mod pm;
pub mod state;

// Server-only: HTTP routing, asset embedding, proxy/cache, OG images, R2, auth,
// background jobs. Gated behind `server` (default) so `cargo install
// --no-default-features --bin xevion` skips them — no `web/build` required.
#[cfg(feature = "server")]
pub mod assets;
#[cfg(feature = "server")]
pub mod auth;
#[cfg(feature = "server")]
pub mod cache;
#[cfg(feature = "server")]
pub mod config;
#[cfg(feature = "server")]
pub mod encoding;
#[cfg(feature = "server")]
pub mod events;
#[cfg(feature = "server")]
pub mod formatter;
#[cfg(feature = "server")]
pub mod github;
#[cfg(feature = "server")]
pub mod health;
#[cfg(feature = "server")]
pub mod http;
#[cfg(feature = "server")]
pub mod icon_cache;
#[cfg(feature = "server")]
pub mod media_processing;
#[cfg(feature = "server")]
pub mod middleware;
#[cfg(feature = "server")]
pub mod og;
#[cfg(feature = "server")]
pub mod proxy;
#[cfg(feature = "server")]
pub mod r2;
#[cfg(feature = "server")]
pub mod routes;
#[cfg(feature = "server")]
pub mod tarpit;
#[cfg(feature = "server")]
pub mod utils;
