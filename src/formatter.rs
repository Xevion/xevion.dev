//! Custom tracing formatter for Railway-compatible structured logging

use nu_ansi_term::Color;
use serde::Serialize;
use serde_json::{Map, Value};
use std::fmt;
use time::macros::format_description;
use time::{format_description::FormatItem, OffsetDateTime};
use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields, FormattedFields};
use tracing_subscriber::registry::LookupSpan;

/// Cached format description for timestamps with 3 subsecond digits (milliseconds)
const TIMESTAMP_FORMAT: &[FormatItem<'static>] =
    format_description!("[hour]:[minute]:[second].[subsecond digits:3]");

/// A custom formatter with enhanced timestamp formatting and colored output
///
/// Provides human-readable output for local development with:
/// - Colored log levels
/// - Timestamp with millisecond precision
/// - Span context with hierarchy
/// - Clean field formatting
pub struct CustomPrettyFormatter;

impl<S, N> FormatEvent<S, N> for CustomPrettyFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let meta = event.metadata();

        // 1) Timestamp (dimmed when ANSI)
        let now = OffsetDateTime::now_utc();
        let formatted_time = now.format(&TIMESTAMP_FORMAT).map_err(|e| {
            eprintln!("Failed to format timestamp: {}", e);
            fmt::Error
        })?;
        write_dimmed(&mut writer, formatted_time)?;
        writer.write_char(' ')?;

        // 2) Colored 5-char level
        write_colored_level(&mut writer, meta.level())?;
        writer.write_char(' ')?;

        // 3) Span scope chain (bold names, fields in braces, dimmed ':')
        if let Some(scope) = ctx.event_scope() {
            let mut saw_any = false;
            for span in scope.from_root() {
                write_bold(&mut writer, span.metadata().name())?;
                saw_any = true;

                write_dimmed(&mut writer, ":")?;

                let ext = span.extensions();
                if let Some(fields) = &ext.get::<FormattedFields<N>>() {
                    if !fields.fields.is_empty() {
                        write_bold(&mut writer, "{")?;
                        writer.write_str(fields.fields.as_str())?;
                        write_bold(&mut writer, "}")?;
                    }
                }
                write_dimmed(&mut writer, ":")?;
            }

            if saw_any {
                writer.write_char(' ')?;
            }
        }

        // 4) Target (dimmed), then a space
        if writer.has_ansi_escapes() {
            write!(writer, "{}: ", Color::DarkGray.paint(meta.target()))?;
        } else {
            write!(writer, "{}: ", meta.target())?;
        }

        // 5) Event fields
        ctx.format_fields(writer.by_ref(), event)?;

        // 6) Newline
        writeln!(writer)
    }
}

/// A custom JSON formatter that flattens fields to root level for Railway
///
/// Outputs logs in Railway-compatible format:
/// ```json
/// {
///   "message": "...",
///   "level": "...",
///   "target": "...",
///   "customAttribute": "..."
/// }
/// ```
///
/// This format allows Railway to:
/// - Parse the `message` field correctly
/// - Filter by `level` and custom attributes using `@attribute:value`
/// - Preserve multi-line logs like stack traces
pub struct CustomJsonFormatter;

impl<S, N> FormatEvent<S, N> for CustomJsonFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let meta = event.metadata();

        #[derive(Serialize)]
        struct EventFields {
            timestamp: String,
            message: String,
            level: String,
            target: String,
            #[serde(flatten)]
            fields: Map<String, Value>,
        }

        let (message, fields) = {
            let mut message: Option<String> = None;
            let mut fields: Map<String, Value> = Map::new();

            struct FieldVisitor<'a> {
                message: &'a mut Option<String>,
                fields: &'a mut Map<String, Value>,
            }

            impl<'a> Visit for FieldVisitor<'a> {
                fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
                    let key = field.name();
                    if key == "message" {
                        *self.message = Some(format!("{:?}", value));
                    } else {
                        self.fields
                            .insert(key.to_string(), Value::String(format!("{:?}", value)));
                    }
                }

                fn record_str(&mut self, field: &Field, value: &str) {
                    let key = field.name();
                    if key == "message" {
                        *self.message = Some(value.to_string());
                    } else {
                        self.fields
                            .insert(key.to_string(), Value::String(value.to_string()));
                    }
                }

                fn record_i64(&mut self, field: &Field, value: i64) {
                    let key = field.name();
                    if key != "message" {
                        self.fields.insert(
                            key.to_string(),
                            Value::Number(serde_json::Number::from(value)),
                        );
                    }
                }

                fn record_u64(&mut self, field: &Field, value: u64) {
                    let key = field.name();
                    if key != "message" {
                        self.fields.insert(
                            key.to_string(),
                            Value::Number(serde_json::Number::from(value)),
                        );
                    }
                }

                fn record_bool(&mut self, field: &Field, value: bool) {
                    let key = field.name();
                    if key != "message" {
                        self.fields.insert(key.to_string(), Value::Bool(value));
                    }
                }
            }

            let mut visitor = FieldVisitor {
                message: &mut message,
                fields: &mut fields,
            };
            event.record(&mut visitor);

            // Collect span information from the span hierarchy
            // Flatten all span fields directly into root level
            if let Some(scope) = ctx.event_scope() {
                for span in scope.from_root() {
                    // Extract span fields by parsing the stored extension data
                    // The fields are stored as a formatted string, so we need to parse them
                    let ext = span.extensions();
                    if let Some(formatted_fields) = ext.get::<FormattedFields<N>>() {
                        let field_str = formatted_fields.fields.as_str();

                        // Parse key=value pairs from the formatted string
                        // Format is typically: key=value key2=value2
                        for pair in field_str.split_whitespace() {
                            if let Some((key, value)) = pair.split_once('=') {
                                // Remove quotes if present
                                let value = value.trim_matches('"').trim_matches('\'');
                                fields.insert(key.to_string(), Value::String(value.to_string()));
                            }
                        }
                    }
                }
            }

            (message, fields)
        };

        let json = EventFields {
            timestamp: OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_else(|_| String::from("1970-01-01T00:00:00Z")),
            message: message.unwrap_or_default(),
            level: meta.level().to_string().to_lowercase(),
            target: meta.target().to_string(),
            fields,
        };

        writeln!(
            writer,
            "{}",
            serde_json::to_string(&json).unwrap_or_else(|_| "{}".to_string())
        )
    }
}

/// Write the verbosity level with colored output
fn write_colored_level(writer: &mut Writer<'_>, level: &Level) -> fmt::Result {
    if writer.has_ansi_escapes() {
        let colored = match *level {
            Level::TRACE => Color::Purple.paint("TRACE"),
            Level::DEBUG => Color::Blue.paint("DEBUG"),
            Level::INFO => Color::Green.paint(" INFO"),
            Level::WARN => Color::Yellow.paint(" WARN"),
            Level::ERROR => Color::Red.paint("ERROR"),
        };
        write!(writer, "{}", colored)
    } else {
        // Right-pad to width 5 for alignment
        match *level {
            Level::TRACE => write!(writer, "{:>5}", "TRACE"),
            Level::DEBUG => write!(writer, "{:>5}", "DEBUG"),
            Level::INFO => write!(writer, "{:>5}", " INFO"),
            Level::WARN => write!(writer, "{:>5}", " WARN"),
            Level::ERROR => write!(writer, "{:>5}", "ERROR"),
        }
    }
}

fn write_dimmed(writer: &mut Writer<'_>, s: impl fmt::Display) -> fmt::Result {
    if writer.has_ansi_escapes() {
        write!(writer, "{}", Color::DarkGray.paint(s.to_string()))
    } else {
        write!(writer, "{}", s)
    }
}

fn write_bold(writer: &mut Writer<'_>, s: impl fmt::Display) -> fmt::Result {
    if writer.has_ansi_escapes() {
        write!(writer, "{}", Color::White.bold().paint(s.to_string()))
    } else {
        write!(writer, "{}", s)
    }
}
