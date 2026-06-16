//! Typed CLI errors with [`miette`] diagnostics.
//!
//! Every `xevion api` subcommand returns [`CliError`]. The variants distinguish
//! the failure modes a user actually cares about — couldn't *connect* vs the
//! request failing, the server's own `{error, code}` body, a bad local file —
//! so `main` can render an actionable diagnostic (with a `help:` line and source
//! chain) and pick a meaningful exit code instead of a flat `Error: …`.

use std::path::PathBuf;

use miette::Diagnostic;
use reqwest::StatusCode;
use snafu::Snafu;

use crate::cli::config::ConfigError;

#[derive(Debug, Snafu, Diagnostic)]
#[snafu(visibility(pub(crate)))]
pub enum CliError {
    /// Config resolution/IO, flattened from [`ConfigError`] (see the `From` impl
    /// below) so the message and forwarded `help` render as a single clean entry
    /// rather than a duplicated wrapper+source pair.
    #[snafu(display("{message}"))]
    #[diagnostic(code(xevion::cli::config))]
    Config {
        message: String,
        #[help]
        advice: Option<String>,
    },

    #[snafu(display("could not connect to {url}"))]
    #[diagnostic(
        code(xevion::cli::connect),
        help(
            "is the server running and reachable? check the target URL with `xevion api targets`"
        )
    )]
    Connect { url: String, source: reqwest::Error },

    #[snafu(display("request to {url} failed"))]
    #[diagnostic(code(xevion::cli::request))]
    Request { url: String, source: reqwest::Error },

    #[snafu(display("server returned {status}: {message}"))]
    #[diagnostic(code(xevion::cli::http))]
    Http {
        status: StatusCode,
        /// Machine-readable `code` from the API error body, when present.
        code: Option<String>,
        message: String,
    },

    #[snafu(display("not authorized for this request"))]
    #[diagnostic(
        code(xevion::cli::unauthorized),
        help("authorize this target first: xevion api login")
    )]
    Unauthorized,

    #[snafu(display("could not decode the server response"))]
    #[diagnostic(code(xevion::cli::decode))]
    Decode { source: reqwest::Error },

    #[snafu(display("could not read {}", path.display()))]
    #[diagnostic(code(xevion::cli::io))]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("{path} is not valid JSON"))]
    #[diagnostic(code(xevion::cli::json))]
    Json {
        path: String,
        source: serde_json::Error,
    },

    #[snafu(display("could not serialize output"))]
    #[diagnostic(code(xevion::cli::serialize))]
    Serialize { source: serde_json::Error },

    #[snafu(display("{message}"))]
    #[diagnostic(code(xevion::cli::invalid))]
    Invalid { message: String },

    #[snafu(display("authorization {reason}"))]
    #[diagnostic(
        code(xevion::cli::approval),
        help("re-run `xevion api login` and approve the request in the browser")
    )]
    Approval { reason: String },
}

impl From<ConfigError> for CliError {
    /// Flatten a [`ConfigError`] into [`CliError::Config`], folding any deeper
    /// source (e.g. the OS error behind a failed read) into the message and
    /// carrying its diagnostic `help` forward.
    fn from(error: ConfigError) -> Self {
        use std::error::Error as _;
        let message = match error.source() {
            Some(source) => format!("{error}: {source}"),
            None => error.to_string(),
        };
        Self::Config {
            message,
            advice: error.help().map(|h| h.to_string()),
        }
    }
}

impl CliError {
    /// Build an [`CliError::Invalid`] from anything printable — the catch-all for
    /// bad user input (status strings, markdown, locators) that has no richer source.
    pub fn invalid(message: impl std::fmt::Display) -> Self {
        Self::Invalid {
            message: message.to_string(),
        }
    }

    /// Map a transport error to [`Connect`](CliError::Connect) when it never
    /// reached the server (connection refused, DNS, TLS, timeout), else
    /// [`Request`](CliError::Request).
    pub fn from_send(url: impl Into<String>, source: reqwest::Error) -> Self {
        let url = url.into();
        if source.is_connect() || source.is_timeout() {
            Self::Connect { url, source }
        } else {
            Self::Request { url, source }
        }
    }

    /// Process exit code, following sysexits conventions where they fit.
    pub const fn exit_code(&self) -> i32 {
        match self {
            Self::Connect { .. } => 69, // EX_UNAVAILABLE
            Self::Unauthorized => 77,   // EX_NOPERM
            Self::Invalid { .. } => 65, // EX_DATAERR
            Self::Io { .. } => 66,      // EX_NOINPUT
            _ => 1,
        }
    }

    /// Machine-readable error object for `--json`, mirroring the API's
    /// `{ error, code }` shape plus the diagnostic `help` when present.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "error": self.to_string(),
            "code": self.code().map(|c| c.to_string()),
            "help": self.help().map(|h| h.to_string()),
        })
    }
}
