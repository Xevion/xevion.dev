//! Shared library for the `xevion` CLI and `xevion-server` binaries.
//!
//! Both binaries link this single crate; the server uses the HTTP/DB/asset
//! modules while the CLI uses [`cli::api`] (plus the shared DTO types in
//! [`db`], [`handlers`], [`pm`], and [`cli_auth`]). See `src/bin/` for the two
//! entry points.

pub mod assets;
pub mod auth;
pub mod cache;
pub mod cli;
pub mod cli_auth;
pub mod config;
pub mod db;
pub mod encoding;
pub mod events;
pub mod formatter;
pub mod github;
pub mod handlers;
pub mod health;
pub mod http;
pub mod icon_cache;
pub mod markdown;
pub mod media_processing;
pub mod middleware;
pub mod og;
pub mod pm;
pub mod proxy;
pub mod r2;
pub mod routes;
pub mod state;
pub mod tarpit;
pub mod utils;
