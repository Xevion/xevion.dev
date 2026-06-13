use nu_ansi_term::{Color, Style};

use crate::db::{ApiAdminProject, ApiSiteSettings, ApiTag, ApiTagWithCount};
use crate::pm::{Doc, Node};

/// Print a success message
pub fn success(msg: &str) {
    println!("{} {}", Color::Green.paint("✓"), msg);
}

/// Print an error message
pub fn error(msg: &str) {
    eprintln!("{} {}", Color::Red.paint("✗"), msg);
}

/// Print an info message
pub fn info(msg: &str) {
    println!("{} {}", Color::Blue.paint("→"), msg);
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

/// Print a content document's blocks as a table (id, type, preview).
pub fn print_blocks(doc: &Doc) {
    let blocks = doc.blocks();
    if blocks.is_empty() {
        info("No content blocks");
        return;
    }

    let header = Style::new().bold().underline();
    let dim = Style::new().dimmed();

    let id_width = blocks
        .iter()
        .filter_map(Node::block_id)
        .map(str::len)
        .max()
        .unwrap_or(2)
        .max(2);
    let type_width = blocks
        .iter()
        .map(|b| b.r#type.len())
        .max()
        .unwrap_or(4)
        .max(4);

    println!(
        "{:id_width$}  {:type_width$}  {}",
        header.paint("ID"),
        header.paint("TYPE"),
        header.paint("PREVIEW"),
    );

    for block in blocks {
        let preview: String = block
            .text_content()
            .lines()
            .next()
            .unwrap_or_default()
            .chars()
            .take(60)
            .collect();
        println!(
            "{:id_width$}  {:type_width$}  {}",
            dim.paint(block.block_id().unwrap_or("—")),
            block.r#type,
            dim.paint(preview),
        );
    }

    println!();
    info(&format!("{} block(s)", blocks.len()));
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

/// Print session info
pub fn print_session(username: &str, api_url: &str) {
    let dim = Style::new().dimmed();
    success(&format!(
        "Logged in as {}",
        Style::new().bold().paint(username)
    ));
    println!("  {} {}", dim.paint("API:"), api_url);
}
