// One-shot import of the legacy Postgres database into D1.
//
//   bun scripts/import-pg.ts --url postgres://... --admin-user admin --out import.sql
//   bunx wrangler d1 execute xevion-dev --remote --file import.sql
//
// Reads every table the new schema keeps and writes SQLite INSERTs in foreign-key order. The admin
// account is recreated with a fresh PBKDF2 hash because Workers cannot verify the old argon2 hash;
// the password comes from XEVION_ADMIN_PASSWORD or an interactive prompt. Only unexpired CLI
// sessions are carried over, so existing `xev_` tokens keep working.
import { SQL } from "bun";
import { writeFileSync } from "node:fs";
import { parseArgs } from "node:util";
import { hashPassword } from "../src/lib/server/crypto.ts";
import { SqlScript } from "./sql.ts";

const { values: args } = parseArgs({
  options: {
    url: { type: "string" },
    out: { type: "string", default: "import.sql" },
    "admin-user": { type: "string", default: "admin" },
  },
});

const url = args.url ?? process.env.DATABASE_URL;
if (!url) throw new Error("pass --url or set DATABASE_URL");

const password =
  process.env.XEVION_ADMIN_PASSWORD ?? prompt("New admin password:");
if (!password || password.length < 8)
  throw new Error("admin password must be 8+ characters");

const pg = new SQL(url);

const script = new SqlScript();
const insert = script.insert.bind(script);

const jsonOrNull = (value: unknown) =>
  value === null || value === undefined
    ? null
    : typeof value === "string"
      ? JSON.parse(value)
      : value;

// The production database already has its admin; this only matters for a fresh database.
insert(
  "users",
  {
    id: 1,
    username: args["admin-user"],
    password_hash: await hashPassword(password),
    created_at: new Date(),
  },
  "INSERT OR IGNORE",
);

const [identity] = await pg`SELECT * FROM site_identity WHERE id = 1`;
if (identity) {
  insert(
    "site_identity",
    {
      id: 1,
      display_name: identity.display_name,
      occupation: identity.occupation,
      bio: identity.bio,
      site_title: identity.site_title,
      resume_r2_key: identity.resume_r2_key,
      resume_uploaded_at: identity.resume_uploaded_at,
      updated_at: identity.updated_at,
    },
    "INSERT OR REPLACE",
  );
}

for (const link of await pg`SELECT * FROM social_links ORDER BY display_order`) {
  insert("social_links", {
    id: link.id,
    platform: link.platform,
    label: link.label,
    value: link.value,
    icon: link.icon,
    visible: link.visible,
    display_order: link.display_order,
  });
}

for (const tag of await pg`SELECT * FROM tags`) {
  insert("tags", {
    id: tag.id,
    slug: tag.slug,
    name: tag.name,
    icon: tag.icon,
    color: tag.color,
    created_at: tag.created_at,
  });
}

for (const p of await pg`SELECT * FROM projects`) {
  insert("projects", {
    id: p.id,
    slug: p.slug,
    name: p.name,
    short_description: p.short_description,
    status: p.status,
    hidden: p.hidden,
    private: p.private,
    github_repo: p.github_repo,
    github_repo_id: p.github_repo_id === null ? null : Number(p.github_repo_id),
    demo_url: p.demo_url,
    project_type: p.project_type,
    accent_color: p.accent_color,
    terminal_cast: jsonOrNull(p.terminal_cast),
    detail_content: jsonOrNull(p.detail_content),
    last_github_activity: p.last_github_activity,
    github_synced_at: p.github_synced_at,
    github_sync_error: p.github_sync_error,
    github_error_count: 0,
    created_at: p.created_at,
    updated_at: p.updated_at,
  });
}

for (const pt of await pg`SELECT * FROM project_tags`) {
  insert("project_tags", { project_id: pt.project_id, tag_id: pt.tag_id });
}

for (const r of await pg`SELECT * FROM project_relations`) {
  insert("project_relations", {
    project_id: r.project_id,
    related_project_id: r.related_project_id,
    position: r.position,
  });
}

for (const m of await pg`SELECT * FROM project_media`) {
  insert("project_media", {
    id: m.id,
    project_id: m.project_id,
    display_order: m.display_order,
    media_type: m.media_type,
    r2_base_path: m.r2_base_path,
    variants: jsonOrNull(m.variants),
    blurhash: m.blurhash,
    metadata: jsonOrNull(m.metadata),
    created_at: m.created_at,
  });
}

const sessions =
  await pg`SELECT * FROM sessions WHERE session_type = 'cli' AND token_hash IS NOT NULL AND expires_at > now()`;
for (const s of sessions) {
  insert("sessions", {
    id: s.id,
    user_id: 1,
    token_hash: s.token_hash,
    kind: "cli",
    label: s.label,
    created_at: s.created_at,
    expires_at: s.expires_at,
    last_active_at: s.last_active_at,
  });
}

await pg.close();

// Imported pages render lazily on first view.
writeFileSync(args.out, script.toString());
console.error(`wrote ${script.statements.length} statements to ${args.out}`);
