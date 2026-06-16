use sqlx::PgPool;

/// Seed the database with sample data
pub async fn run(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("Seeding database...");

    // Clear existing data (tags will cascade delete project_tags and tag_cooccurrence)
    sqlx::query("DELETE FROM social_links")
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM tags").execute(pool).await?;
    sqlx::query("DELETE FROM projects").execute(pool).await?;

    // Seed site identity
    sqlx::query(
        r"
        INSERT INTO site_identity (id, display_name, occupation, bio, site_title)
        VALUES (1, $1, $2, $3, $4)
        ON CONFLICT (id) DO UPDATE SET
            display_name = EXCLUDED.display_name,
            occupation = EXCLUDED.occupation,
            bio = EXCLUDED.bio,
            site_title = EXCLUDED.site_title
        ",
    )
    .bind("Ryan Walters")
    .bind("Full-Stack Software Engineer")
    .bind("A fanatical software engineer with expertise and passion for sound, scalable and high-performance applications. I'm always working on something new.\nSometimes innovative — sometimes crazy.")
    .bind("Xevion.dev")
    .execute(pool)
    .await?;

    println!("  Seeded site identity");

    // Seed social links
    let social_links = vec![
        (
            "github",
            "GitHub",
            "https://github.com/Xevion",
            "simple-icons:github",
            1,
        ),
        (
            "linkedin",
            "LinkedIn",
            "https://linkedin.com/in/ryancwalters",
            "simple-icons:linkedin",
            2,
        ),
        ("discord", "Discord", ".xevion", "simple-icons:discord", 3),
        (
            "email",
            "Email",
            "xevion@xevion.dev",
            "material-symbols:mail-rounded",
            4,
        ),
    ];

    for (platform, label, value, icon, order) in &social_links {
        sqlx::query(
            r"
            INSERT INTO social_links (platform, label, value, icon, visible, display_order)
            VALUES ($1, $2, $3, $4, true, $5)
            ",
        )
        .bind(platform)
        .bind(label)
        .bind(value)
        .bind(icon)
        .bind(order)
        .execute(pool)
        .await?;
    }

    println!("  Seeded {} social links", social_links.len());

    // Seed projects matching production data
    let projects = vec![
        (
            "xevion-dev",
            "xevion.dev",
            "A dual-process portfolio website built with Rust and SvelteKit",
            "active",
            Some("xevion/xevion.dev"),
            Some("https://xevion.dev"),
        ),
        (
            "rdap",
            "rdap",
            "Modern RDAP query client for domain and IP lookups",
            "active",
            Some("Xevion/rdap"),
            Some("https://rdap.xevion.dev"),
        ),
        (
            "byte-me",
            "byte-me",
            "Cross-platform media bitrate visualizer built with Tauri",
            "active",
            Some("Xevion/byte-me"),
            None,
        ),
        (
            "pac-man",
            "pac-man",
            "Classic Pac-Man arcade game clone, playable in the browser",
            "active",
            Some("Xevion/Pac-Man"),
            Some("https://pacman.xevion.dev"),
        ),
        (
            "rebinded",
            "Rebinded",
            "Cross-platform key remapping daemon with per-app context",
            "active",
            Some("Xevion/rebinded"),
            None,
        ),
        (
            "the-office-quotes",
            "The Office Quotes",
            "Search and browse quotes from The Office TV series",
            "active",
            Some("Xevion/the-office"),
            Some("https://the-office.xevion.dev"),
        ),
        (
            "grain",
            "grain",
            "SVG-based film grain noise and radial gradient effects",
            "active",
            Some("Xevion/grain"),
            Some("https://grain.xevion.dev"),
        ),
        (
            "dynamic-preauth",
            "dynamic-preauth",
            "Server-side executable pre-authentication proof of concept",
            "active",
            Some("Xevion/dynamic-preauth"),
            Some("https://dynamic-preauth.xevion.dev"),
        ),
        (
            "rustdoc-mcp",
            "rustdoc-mcp",
            "intelligent MCP server providing access to Rust documentation",
            "active",
            Some("Xevion/rustdoc-mcp"),
            None,
        ),
    ];

    let project_count = projects.len();

    for (slug, name, short_desc, status, repo, demo) in projects {
        sqlx::query(
            r"
            INSERT INTO projects (slug, name, short_description, status, github_repo, demo_url)
            VALUES ($1, $2, $3, $4::project_status, $5, $6)
            ",
        )
        .bind(slug)
        .bind(name)
        .bind(short_desc)
        .bind(status)
        .bind(repo)
        .bind(demo)
        .execute(pool)
        .await?;
    }

    println!("  Seeded {project_count} projects");

    // Seed tags matching production data
    let tags = vec![
        ("astro", "Astro", "simple-icons:astro", "FF5D01"),
        ("cli", "CLI", "lucide:terminal", "22C55E"),
        ("game", "Game", "lucide:gamepad-2", "EF4444"),
        ("mcp", "MCP", "lucide:plug", "8B5CF6"),
        ("nextjs", "Nextjs", "simple-icons:nextdotjs", "000000"),
        ("preact", "Preact", "simple-icons:preact", "673AB8"),
        ("python", "Python", "simple-icons:python", "3776AB"),
        ("react", "React", "simple-icons:react", "61DAFB"),
        ("rust", "Rust", "simple-icons:rust", "DEA584"),
        ("security", "Security", "lucide:shield", "F59E0B"),
        ("sveltekit", "SvelteKit", "simple-icons:svelte", "FF3E00"),
        ("tauri", "Tauri", "simple-icons:tauri", "24C8DB"),
        (
            "typescript",
            "TypeScript",
            "simple-icons:typescript",
            "3178C6",
        ),
        ("vue", "Vue", "simple-icons:vuedotjs", "4FC08D"),
        (
            "webassembly",
            "WebAssembly",
            "simple-icons:webassembly",
            "654FF0",
        ),
        (
            "web-development",
            "Web Development",
            "lucide:globe",
            "3B82F6",
        ),
    ];

    let mut tag_ids = std::collections::HashMap::new();

    for (slug, name, icon, color) in tags {
        let result = sqlx::query!(
            r#"
            INSERT INTO tags (slug, name, icon, color)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
            slug,
            name,
            icon,
            color
        )
        .fetch_one(pool)
        .await?;

        tag_ids.insert(slug, result.id);
    }

    println!("  Seeded {} tags", tag_ids.len());

    // Associate tags with projects (matching production)
    let project_tag_associations = vec![
        ("xevion-dev", vec!["cli", "rust", "sveltekit", "typescript"]),
        ("rdap", vec!["nextjs", "react", "typescript"]),
        ("byte-me", vec!["rust", "tauri", "typescript"]),
        ("pac-man", vec!["game", "rust", "webassembly"]),
        ("rebinded", vec!["cli", "rust"]),
        ("the-office-quotes", vec!["python", "vue"]),
        ("grain", vec!["preact", "typescript"]),
        (
            "dynamic-preauth",
            vec!["astro", "rust", "security", "typescript"],
        ),
        ("rustdoc-mcp", vec!["cli", "mcp", "rust"]),
    ];

    let mut association_count = 0;

    for (project_slug, tag_slugs) in project_tag_associations {
        let project_id = sqlx::query!("SELECT id FROM projects WHERE slug = $1", project_slug)
            .fetch_one(pool)
            .await?
            .id;

        for tag_slug in tag_slugs {
            if let Some(&tag_id) = tag_ids.get(tag_slug) {
                sqlx::query!(
                    "INSERT INTO project_tags (project_id, tag_id) VALUES ($1, $2)",
                    project_id,
                    tag_id
                )
                .execute(pool)
                .await?;

                association_count += 1;
            }
        }
    }

    println!("  Created {association_count} project-tag associations");

    // Recalculate tag cooccurrence
    sqlx::query!("DELETE FROM tag_cooccurrence")
        .execute(pool)
        .await?;

    sqlx::query!(
        r#"
        INSERT INTO tag_cooccurrence (tag_a, tag_b, count)
        SELECT
            LEAST(t1.tag_id, t2.tag_id) as tag_a,
            GREATEST(t1.tag_id, t2.tag_id) as tag_b,
            COUNT(*)::int as count
        FROM project_tags t1
        JOIN project_tags t2 ON t1.project_id = t2.project_id
        WHERE t1.tag_id < t2.tag_id
        GROUP BY tag_a, tag_b
        HAVING COUNT(*) > 0
        "#
    )
    .execute(pool)
    .await?;

    println!("  Recalculated tag cooccurrence");
    println!("Database seeded successfully!");

    Ok(())
}
