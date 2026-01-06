use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")?;
    let pool = PgPool::connect(&database_url).await?;

    println!("🌱 Seeding database...");

    // Clear existing data
    sqlx::query("DELETE FROM projects").execute(&pool).await?;

    // Seed projects with diverse data
    let projects = vec![
        (
            "xevion-dev",
            "xevion.dev",
            "Personal portfolio site with fuzzy tag discovery and ISR caching",
            "active",
            Some("Xevion/xevion.dev"),
            None,
            10,
            Some("fa-globe"),
        ),
        (
            "contest",
            "Contest",
            "Archive and analysis platform for competitive programming problems",
            "active",
            Some("Xevion/contest"),
            Some("https://contest.xevion.dev"),
            9,
            Some("fa-trophy"),
        ),
        (
            "reforge",
            "Reforge",
            "Rust library for parsing and manipulating Replay files from Rocket League",
            "maintained",
            Some("Xevion/reforge"),
            None,
            8,
            Some("fa-file-code"),
        ),
        (
            "algorithms",
            "Algorithms",
            "Collection of algorithm implementations and data structures in Python",
            "archived",
            Some("Xevion/algorithms"),
            None,
            5,
            Some("fa-brain"),
        ),
        (
            "wordplay",
            "WordPlay",
            "Interactive word game with real-time multiplayer using WebSockets",
            "maintained",
            Some("Xevion/wordplay"),
            Some("https://wordplay.example.com"),
            7,
            Some("fa-gamepad"),
        ),
        (
            "dotfiles",
            "Dotfiles",
            "Personal configuration files and development environment setup scripts",
            "active",
            Some("Xevion/dotfiles"),
            None,
            6,
            Some("fa-terminal"),
        ),
    ];

    let project_count = projects.len();

    for (slug, title, desc, status, repo, demo, priority, icon) in projects {
        sqlx::query(
            r#"
            INSERT INTO projects (slug, title, description, status, github_repo, demo_url, priority, icon)
            VALUES ($1, $2, $3, $4::project_status, $5, $6, $7, $8)
            "#,
        )
        .bind(slug)
        .bind(title)
        .bind(desc)
        .bind(status)
        .bind(repo)
        .bind(demo)
        .bind(priority)
        .bind(icon)
        .execute(&pool)
        .await?;
    }

    println!("✅ Seeded {} projects", project_count);

    Ok(())
}
