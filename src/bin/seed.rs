use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")?;
    let pool = PgPool::connect(&database_url).await?;

    println!("🌱 Seeding database...");

    // Clear existing data (tags will cascade delete project_tags and tag_cooccurrence)
    sqlx::query("DELETE FROM tags").execute(&pool).await?;
    sqlx::query("DELETE FROM projects").execute(&pool).await?;

    // Seed projects with diverse data
    let projects = vec![
        (
            "xevion-dev",
            "xevion.dev",
            "Personal portfolio and project showcase",
            "Personal portfolio site with fuzzy tag discovery and ISR caching",
            "active",
            Some("Xevion/xevion.dev"),
            None,
        ),
        (
            "contest",
            "Contest",
            "Competitive programming archive",
            "Archive and analysis platform for competitive programming problems",
            "active",
            Some("Xevion/contest"),
            Some("https://contest.xevion.dev"),
        ),
        (
            "reforge",
            "Reforge",
            "Rocket League replay parser",
            "Rust library for parsing and manipulating Replay files from Rocket League",
            "maintained",
            Some("Xevion/reforge"),
            None,
        ),
        (
            "algorithms",
            "Algorithms",
            "Algorithm implementations in Python",
            "Collection of algorithm implementations and data structures in Python",
            "archived",
            Some("Xevion/algorithms"),
            None,
        ),
        (
            "wordplay",
            "WordPlay",
            "Real-time multiplayer word game",
            "Interactive word game with real-time multiplayer using WebSockets",
            "maintained",
            Some("Xevion/wordplay"),
            Some("https://wordplay.example.com"),
        ),
        (
            "dotfiles",
            "Dotfiles",
            "Development environment configs",
            "Personal configuration files and development environment setup scripts",
            "active",
            Some("Xevion/dotfiles"),
            None,
        ),
    ];

    let project_count = projects.len();

    for (slug, name, short_desc, desc, status, repo, demo) in projects {
        sqlx::query(
            r#"
            INSERT INTO projects (slug, name, short_description, description, status, github_repo, demo_url)
            VALUES ($1, $2, $3, $4, $5::project_status, $6, $7)
            "#,
        )
        .bind(slug)
        .bind(name)
        .bind(short_desc)
        .bind(desc)
        .bind(status)
        .bind(repo)
        .bind(demo)
        .execute(&pool)
        .await?;
    }

    println!("✅ Seeded {} projects", project_count);

    // Seed tags
    let tags = vec![
        ("rust", "Rust", "simple-icons:rust"),
        ("python", "Python", "simple-icons:python"),
        ("typescript", "TypeScript", "simple-icons:typescript"),
        ("javascript", "JavaScript", "simple-icons:javascript"),
        ("web", "Web", "lucide:globe"),
        ("cli", "CLI", "lucide:terminal"),
        ("library", "Library", "lucide:package"),
        ("game", "Game", "lucide:gamepad-2"),
        ("data-structures", "Data Structures", "lucide:database"),
        ("algorithms", "Algorithms", "lucide:cpu"),
        ("multiplayer", "Multiplayer", "lucide:users"),
        ("config", "Config", "lucide:settings"),
    ];

    let mut tag_ids = std::collections::HashMap::new();

    for (slug, name, icon) in tags {
        let result = sqlx::query!(
            r#"
            INSERT INTO tags (slug, name, icon)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
            slug,
            name,
            icon
        )
        .fetch_one(&pool)
        .await?;

        tag_ids.insert(slug, result.id);
    }

    println!("✅ Seeded {} tags", tag_ids.len());

    // Associate tags with projects
    let project_tag_associations = vec![
        // xevion-dev
        ("xevion-dev", vec!["rust", "web", "typescript"]),
        // Contest
        (
            "contest",
            vec!["python", "web", "algorithms", "data-structures"],
        ),
        // Reforge
        ("reforge", vec!["rust", "library", "game"]),
        // Algorithms
        (
            "algorithms",
            vec!["python", "algorithms", "data-structures"],
        ),
        // WordPlay
        (
            "wordplay",
            vec!["typescript", "javascript", "web", "game", "multiplayer"],
        ),
        // Dotfiles
        ("dotfiles", vec!["config", "cli"]),
    ];

    let mut association_count = 0;

    for (project_slug, tag_slugs) in project_tag_associations {
        let project_id = sqlx::query!("SELECT id FROM projects WHERE slug = $1", project_slug)
            .fetch_one(&pool)
            .await?
            .id;

        for tag_slug in tag_slugs {
            if let Some(&tag_id) = tag_ids.get(tag_slug) {
                sqlx::query!(
                    "INSERT INTO project_tags (project_id, tag_id) VALUES ($1, $2)",
                    project_id,
                    tag_id
                )
                .execute(&pool)
                .await?;

                association_count += 1;
            }
        }
    }

    println!("✅ Created {} project-tag associations", association_count);

    // Recalculate tag cooccurrence
    sqlx::query!("DELETE FROM tag_cooccurrence")
        .execute(&pool)
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
    .execute(&pool)
    .await?;

    println!("✅ Recalculated tag cooccurrence");

    Ok(())
}
