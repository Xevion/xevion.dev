use nu_ansi_term::{Color, Style};
use serde::Serialize;
use snafu::ResultExt;

use crate::cli::error::{CliError, SerializeSnafu};
use crate::db::{ApiAdminProject, ApiSiteSettings, ApiTag, ApiTagWithCount};
use crate::pm::{Doc, Node};

// Status, progress, and confirmation lines are diagnostics: they go to stderr so
// stdout carries only data (tables in human mode, JSON in `--json` mode) and
// every command stays cleanly pipeable.

/// Print a success message (stderr).
pub fn success(msg: &str) {
    eprintln!("{} {}", Color::Green.paint("✓"), msg);
}

/// Print an error/warning message (stderr).
pub fn error(msg: &str) {
    eprintln!("{} {}", Color::Red.paint("✗"), msg);
}

/// Print an info/progress message (stderr).
pub fn info(msg: &str) {
    eprintln!("{} {}", Color::Blue.paint("→"), msg);
}

/// Pretty-print a value as JSON to stdout (the machine-readable data channel).
pub fn print_json<T: Serialize>(value: &T) -> Result<(), CliError> {
    let rendered = serde_json::to_string_pretty(value).context(SerializeSnafu)?;
    println!("{rendered}");
    Ok(())
}

/// Print a project in formatted output
pub fn print_project(project: &ApiAdminProject) {
    let header = Style::new().bold();
    let dim = Style::new().dimmed();

    println!("{}", header.paint(&project.project.name));
    println!("  {} {}", dim.paint("ID:"), project.project.id);
    println!("  {} {}", dim.paint("Slug:"), project.project.slug);
    println!(
        "  {} {}",
        dim.paint("Status:"),
        format_status(project.status)
    );
    println!(
        "  {} {}",
        dim.paint("Description:"),
        project.project.short_description
    );

    if let Some(ref repo) = project.github_repo {
        println!("  {} {}", dim.paint("GitHub:"), repo);
    }
    if let Some(ref url) = project.demo_url {
        println!("  {} {}", dim.paint("Demo:"), url);
    }

    if !project.tags.is_empty() {
        let tags: Vec<_> = project.tags.iter().map(|t| t.slug.as_str()).collect();
        println!("  {} {}", dim.paint("Tags:"), tags.join(", "));
    }

    println!(
        "  {} {}",
        dim.paint("Last Activity:"),
        project.last_activity
    );
}

/// Print a list of projects in table format
pub fn print_projects_table(projects: &[ApiAdminProject]) {
    if projects.is_empty() {
        info("No projects found");
        return;
    }

    let header = Style::new().bold().underline();
    let dim = Style::new().dimmed();

    // Calculate column widths
    let name_width = projects
        .iter()
        .map(|p| p.project.name.len())
        .max()
        .unwrap_or(4)
        .max(4);
    let slug_width = projects
        .iter()
        .map(|p| p.project.slug.len())
        .max()
        .unwrap_or(4)
        .max(4);

    // Header
    println!(
        "{:name_width$}  {:slug_width$}  {:10}  {}",
        header.paint("NAME"),
        header.paint("SLUG"),
        header.paint("STATUS"),
        header.paint("TAGS"),
    );

    // Rows
    for project in projects {
        let tags: Vec<_> = project.tags.iter().map(|t| t.slug.as_str()).collect();
        let tags_str = if tags.is_empty() {
            dim.paint("-").to_string()
        } else {
            tags.join(", ")
        };

        println!(
            "{:name_width$}  {:slug_width$}  {:10}  {}",
            project.project.name,
            dim.paint(&project.project.slug),
            format_status(project.status),
            tags_str,
        );
    }

    println!();
    info(&format!("{} project(s)", projects.len()));
}

/// One rendered row of the block outline. Kept apart from painting so the
/// layout — ids, paths, tree indentation, preview truncation — is unit testable
/// without ANSI escapes in the assertions.
struct BlockRow {
    /// Stable block id, absent on content that has never been stamped.
    id: Option<String>,
    /// Positional path (`.3.0`) — the always-available addressing handle.
    path: String,
    /// Type name, indented two spaces per level of depth to draw the tree.
    type_label: String,
    /// First line of the block's own text, already truncated.
    preview: String,
}

/// Longest first line shown in the PREVIEW column before it is ellipsized.
const PREVIEW_MAX: usize = 60;

/// Assemble the printable rows for a document's block outline. Pure (returns
/// owned plain strings), so tests assert structure directly.
fn block_rows(doc: &Doc) -> Vec<BlockRow> {
    doc.outline()
        .into_iter()
        .map(|(path, node)| {
            let depth = path.indices().len().saturating_sub(1);
            BlockRow {
                id: node.block_id().map(str::to_string),
                path: path.to_string(),
                type_label: format!("{}{}", "  ".repeat(depth), node.r#type),
                preview: preview_of(node),
            }
        })
        .collect()
}

/// The first line of a block's own text, cut to [`PREVIEW_MAX`] chars with a
/// trailing ellipsis when it was actually cut.
fn preview_of(node: &Node) -> String {
    let text = node.direct_text();
    let first = text.lines().next().unwrap_or_default();
    let head: String = first.chars().take(PREVIEW_MAX).collect();
    if first.chars().count() > PREVIEW_MAX {
        format!("{head}…")
    } else {
        head
    }
}

/// Paint `text` in `style`, then pad to a *visible* width of `width`. The pad is
/// computed from the unpainted char count — `{:width$}` on a painted string
/// would count the ANSI escapes and misalign the column. Never truncates.
fn padded(text: &str, style: Style, width: usize) -> String {
    let pad = width.saturating_sub(text.chars().count());
    format!("{}{}", style.paint(text), " ".repeat(pad))
}

/// Print a content document as an indented block outline. Two flush-left
/// identifier columns lead each row: the stable ID (the handle that survives
/// reordering, `-` until content is first stamped) and the positional PATH
/// (`content replace <ref> .3.0 …`). The depth-indented TYPE draws the tree and
/// PREVIEW shows the block's text.
pub fn print_blocks(doc: &Doc) {
    let rows = block_rows(doc);
    if rows.is_empty() {
        info("No content blocks");
        return;
    }

    let header = Style::new().bold().underline();
    let dim = Style::new().dimmed();
    let path_style = Color::Cyan.normal();
    let plain = Style::new();

    let id_cell = |row: &BlockRow| {
        row.id
            .as_deref()
            .map_or_else(|| "-".to_string(), |id| format!("#{id}"))
    };

    // Each column width carries a two-space gutter; PREVIEW is last and unpadded.
    let gutter = 2;
    let id_width = column_width(rows.iter().map(|r| id_cell(r).chars().count()), 2) + gutter;
    let path_width = column_width(rows.iter().map(|r| r.path.chars().count()), 4) + gutter;
    let type_width = column_width(rows.iter().map(|r| r.type_label.chars().count()), 4) + gutter;

    println!(
        "{}{}{}{}",
        padded("ID", header, id_width),
        padded("PATH", header, path_width),
        padded("TYPE", header, type_width),
        header.paint("PREVIEW"),
    );

    for row in &rows {
        println!(
            "{}{}{}{}",
            padded(&id_cell(row), dim, id_width),
            padded(&row.path, path_style, path_width),
            padded(&row.type_label, plain, type_width),
            dim.paint(&row.preview),
        );
    }

    println!();
    info(&format!("{} block(s)", rows.len()));
}

/// Widest cell in a column, floored at `min` (so a column never narrows past
/// its header label).
fn column_width(cells: impl Iterator<Item = usize>, min: usize) -> usize {
    cells.max().unwrap_or(min).max(min)
}

/// Print a tag in formatted output
pub fn print_tag(tag: &ApiTag) {
    let header = Style::new().bold();
    let dim = Style::new().dimmed();

    println!("{}", header.paint(&tag.name));
    println!("  {} {}", dim.paint("ID:"), tag.id);
    println!("  {} {}", dim.paint("Slug:"), tag.slug);

    if let Some(ref icon) = tag.icon {
        println!("  {} {}", dim.paint("Icon:"), icon);
    }
    if let Some(ref color) = tag.color {
        println!("  {} #{}", dim.paint("Color:"), color);
    }
}

/// Print a list of tags in table format
pub fn print_tags_table(tags: &[ApiTagWithCount]) {
    if tags.is_empty() {
        info("No tags found");
        return;
    }

    let header = Style::new().bold().underline();
    let dim = Style::new().dimmed();

    // Calculate column widths
    let name_width = tags
        .iter()
        .map(|t| t.tag.name.len())
        .max()
        .unwrap_or(4)
        .max(4);
    let slug_width = tags
        .iter()
        .map(|t| t.tag.slug.len())
        .max()
        .unwrap_or(4)
        .max(4);

    // Header
    println!(
        "{:name_width$}  {:slug_width$}  {:8}  {:20}  {}",
        header.paint("NAME"),
        header.paint("SLUG"),
        header.paint("PROJECTS"),
        header.paint("ICON"),
        header.paint("COLOR"),
    );

    // Rows
    for tag in tags {
        let icon = tag.tag.icon.as_deref().unwrap_or("-");
        let color = tag
            .tag
            .color
            .as_ref()
            .map_or_else(|| "-".to_string(), |c| format!("#{c}"));

        println!(
            "{:name_width$}  {:slug_width$}  {:8}  {:20}  {}",
            tag.tag.name,
            dim.paint(&tag.tag.slug),
            tag.project_count,
            dim.paint(icon),
            dim.paint(&color),
        );
    }

    println!();
    info(&format!("{} tag(s)", tags.len()));
}

/// Print site settings in formatted output
pub fn print_settings(settings: &ApiSiteSettings) {
    let header = Style::new().bold();
    let dim = Style::new().dimmed();

    println!("{}", header.paint("Site Identity"));
    println!(
        "  {} {}",
        dim.paint("Display Name:"),
        settings.identity.display_name
    );
    println!(
        "  {} {}",
        dim.paint("Occupation:"),
        settings.identity.occupation
    );
    println!("  {} {}", dim.paint("Bio:"), settings.identity.bio);
    println!(
        "  {} {}",
        dim.paint("Site Title:"),
        settings.identity.site_title
    );

    if !settings.social_links.is_empty() {
        println!();
        println!("{}", header.paint("Social Links"));
        for link in &settings.social_links {
            let visibility = if link.visible { "" } else { " (hidden)" };
            println!(
                "  {} {}: {}{}",
                dim.paint(format!("[{}]", link.display_order)),
                link.label,
                link.value,
                dim.paint(visibility)
            );
        }
    }
}

/// Format project status with color
fn format_status(status: crate::db::ProjectStatus) -> String {
    use crate::db::ProjectStatus;
    let label = match status {
        ProjectStatus::Active => "active",
        ProjectStatus::Maintained => "maintained",
        ProjectStatus::Archived => "archived",
        ProjectStatus::Hidden => "hidden",
    };
    let color = match status {
        ProjectStatus::Active => Color::Green,
        ProjectStatus::Maintained => Color::Blue,
        ProjectStatus::Archived => Color::Yellow,
        ProjectStatus::Hidden => Color::Red,
    };
    color.paint(label).to_string()
}

/// Print session info (stderr — this is status, not pipeable data)
pub fn print_session(username: &str, api_url: &str) {
    let dim = Style::new().dimmed();
    success(&format!(
        "Logged in as {}",
        Style::new().bold().paint(username)
    ));
    eprintln!("  {} {}", dim.paint("API:"), api_url);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pm::Doc;
    use serde_json::json;

    fn doc(value: &serde_json::Value) -> Doc {
        Doc::from_stored(Some(value))
    }

    #[test]
    fn block_rows_reports_id_path_type_and_tree_indentation() {
        let d = doc(&json!({
            "type": "doc",
            "content": [
                { "type": "paragraph", "attrs": { "id": "aaaa1111" },
                  "content": [{ "type": "text", "text": "hello" }] },
                { "type": "bulletList", "content": [
                    { "type": "listItem", "content": [
                        { "type": "paragraph",
                          "content": [{ "type": "text", "text": "nested" }] }
                    ] }
                ] }
            ]
        }));
        let rows = block_rows(&d);
        assert_eq!(rows.len(), 4);

        assert_eq!(rows[0].path, ".0");
        assert_eq!(rows[0].id.as_deref(), Some("aaaa1111"));
        assert_eq!(rows[0].type_label, "paragraph");
        assert_eq!(rows[0].preview, "hello");

        assert_eq!(rows[1].path, ".1");
        assert_eq!(rows[1].id, None);
        assert_eq!(rows[1].type_label, "bulletList");

        assert_eq!(rows[2].path, ".1.0");
        assert_eq!(rows[2].type_label, "  listItem");

        assert_eq!(rows[3].path, ".1.0.0");
        assert_eq!(rows[3].type_label, "    paragraph");
        assert_eq!(rows[3].preview, "nested");
    }

    #[test]
    fn preview_truncates_long_first_line_with_ellipsis() {
        let long = "x".repeat(80);
        let d = doc(&json!({
            "type": "doc",
            "content": [
                { "type": "paragraph", "content": [{ "type": "text", "text": long }] }
            ]
        }));
        let rows = block_rows(&d);
        assert_eq!(rows[0].preview.chars().count(), 61);
        assert!(rows[0].preview.ends_with('…'));
    }

    #[test]
    fn padded_fills_to_visible_width() {
        // A plain style emits no escapes, so the visible width is exact.
        assert_eq!(padded("ab", Style::new(), 5), "ab   ");
        // Already at/over width: no padding, never truncates.
        assert_eq!(padded("abcde", Style::new(), 5), "abcde");
        assert_eq!(padded("abcdef", Style::new(), 5), "abcdef");
    }
}
