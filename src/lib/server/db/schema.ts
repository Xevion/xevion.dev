import { sql } from "drizzle-orm";
import {
  check,
  index,
  integer,
  primaryKey,
  sqliteTable,
  text,
  uniqueIndex,
} from "drizzle-orm/sqlite-core";
import type { MediaMetadata, TerminalCast } from "#lib/bindings/index.ts";
import type { StoredVariants } from "#lib/server/media-variants.ts";
import type { TocItem } from "#lib/tiptap/toc.ts";

const uuid = () =>
  text("id")
    .primaryKey()
    .$defaultFn(() => crypto.randomUUID());

const createdAt = () =>
  integer("created_at", { mode: "timestamp" })
    .notNull()
    .$defaultFn(() => new Date());

export const PROJECT_STATUSES = ["active", "maintained", "archived"] as const;

export const projects = sqliteTable(
  "projects",
  {
    id: uuid(),
    slug: text("slug").notNull().unique(),
    name: text("name").notNull(),
    shortDescription: text("short_description").notNull().default(""),
    status: text("status", { enum: PROJECT_STATUSES })
      .notNull()
      .default("active"),
    hidden: integer("hidden", { mode: "boolean" }).notNull().default(false),
    private: integer("private", { mode: "boolean" }).notNull().default(false),
    githubRepo: text("github_repo"),
    githubRepoId: integer("github_repo_id"),
    demoUrl: text("demo_url"),
    projectType: text("project_type"),
    accentColor: text("accent_color"),
    terminalCast: text("terminal_cast", { mode: "json" }).$type<TerminalCast>(),
    detailContent: text("detail_content", { mode: "json" }).$type<unknown>(),
    // Rendered at save time so a page view never runs TipTap or Shiki.
    renderedHtml: text("rendered_html"),
    toc: text("toc", { mode: "json" }).$type<TocItem[]>(),
    lastGithubActivity: integer("last_github_activity", { mode: "timestamp" }),
    githubSyncedAt: integer("github_synced_at", { mode: "timestamp" }),
    githubSyncError: text("github_sync_error"),
    githubNextSyncAt: integer("github_next_sync_at", { mode: "timestamp" }),
    githubErrorCount: integer("github_error_count").notNull().default(0),
    githubEtag: text("github_etag"),
    githubEtagValue: integer("github_etag_value", { mode: "timestamp" }),
    createdAt: createdAt(),
    // Editorial edits only: GitHub sync must never bump it, because OG cache-busting and the
    // sitemap's lastmod derive from it. Set through touchProject(), never $onUpdate.
    updatedAt: integer("updated_at", { mode: "timestamp" })
      .notNull()
      .$defaultFn(() => new Date()),
  },
  (t) => [index("idx_projects_next_sync").on(t.githubNextSyncAt)],
);

export const tags = sqliteTable(
  "tags",
  {
    id: uuid(),
    slug: text("slug").notNull().unique(),
    name: text("name").notNull(),
    icon: text("icon"),
    color: text("color"),
    createdAt: createdAt(),
  },
  (t) => [uniqueIndex("idx_tags_name_lower").on(sql`lower(${t.name})`)],
);

export const projectTags = sqliteTable(
  "project_tags",
  {
    projectId: text("project_id")
      .notNull()
      .references(() => projects.id, { onDelete: "cascade" }),
    tagId: text("tag_id")
      .notNull()
      .references(() => tags.id, { onDelete: "cascade" }),
  },
  (t) => [
    primaryKey({ columns: [t.projectId, t.tagId] }),
    index("idx_project_tags_tag").on(t.tagId),
  ],
);

export const projectRelations = sqliteTable(
  "project_relations",
  {
    projectId: text("project_id")
      .notNull()
      .references(() => projects.id, { onDelete: "cascade" }),
    relatedProjectId: text("related_project_id")
      .notNull()
      .references(() => projects.id, { onDelete: "cascade" }),
    position: integer("position").notNull(),
  },
  (t) => [
    primaryKey({ columns: [t.projectId, t.relatedProjectId] }),
    index("idx_project_relations_order").on(t.projectId, t.position),
    check(
      "project_relations_no_self",
      sql`${t.projectId} <> ${t.relatedProjectId}`,
    ),
  ],
);

export const MEDIA_TYPES = ["image", "video"] as const;

export const projectMedia = sqliteTable(
  "project_media",
  {
    id: uuid(),
    projectId: text("project_id")
      .notNull()
      .references(() => projects.id, { onDelete: "cascade" }),
    displayOrder: integer("display_order").notNull().default(0),
    mediaType: text("media_type", { enum: MEDIA_TYPES }).notNull(),
    r2BasePath: text("r2_base_path").notNull(),
    variants: text("variants", { mode: "json" })
      .$type<StoredVariants>()
      .notNull(),
    blurhash: text("blurhash"),
    metadata: text("metadata", { mode: "json" }).$type<MediaMetadata>(),
    createdAt: createdAt(),
  },
  (t) => [
    uniqueIndex("idx_project_media_order").on(t.projectId, t.displayOrder),
  ],
);

export const siteIdentity = sqliteTable(
  "site_identity",
  {
    id: integer("id").primaryKey(),
    displayName: text("display_name").notNull(),
    occupation: text("occupation").notNull(),
    bio: text("bio").notNull(),
    siteTitle: text("site_title").notNull(),
    resumeR2Key: text("resume_r2_key"),
    resumeUploadedAt: integer("resume_uploaded_at", { mode: "timestamp" }),
    updatedAt: integer("updated_at", { mode: "timestamp" })
      .notNull()
      .$defaultFn(() => new Date())
      .$onUpdate(() => new Date()),
  },
  (t) => [check("site_identity_single_row", sql`${t.id} = 1`)],
);

export const socialLinks = sqliteTable(
  "social_links",
  {
    id: uuid(),
    platform: text("platform").notNull(),
    label: text("label").notNull(),
    value: text("value").notNull(),
    icon: text("icon").notNull(),
    visible: integer("visible", { mode: "boolean" }).notNull().default(true),
    displayOrder: integer("display_order").notNull().default(0),
  },
  (t) => [index("idx_social_links_order").on(t.displayOrder)],
);

export const users = sqliteTable("users", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  username: text("username").notNull().unique(),
  passwordHash: text("password_hash").notNull(),
  createdAt: createdAt(),
});

export const SESSION_KINDS = ["browser", "cli"] as const;

export const sessions = sqliteTable(
  "sessions",
  {
    id: uuid(),
    userId: integer("user_id")
      .notNull()
      .references(() => users.id, { onDelete: "cascade" }),
    // SHA-256 hex of the bearer value. The cookie or token itself is never stored.
    tokenHash: text("token_hash").notNull().unique(),
    kind: text("kind", { enum: SESSION_KINDS }).notNull(),
    label: text("label"),
    createdAt: createdAt(),
    expiresAt: integer("expires_at", { mode: "timestamp" }).notNull(),
    lastActiveAt: integer("last_active_at", { mode: "timestamp" })
      .notNull()
      .$defaultFn(() => new Date()),
  },
  (t) => [
    index("idx_sessions_user").on(t.userId),
    index("idx_sessions_expires").on(t.expiresAt),
  ],
);

export const DEVICE_REQUEST_STATUSES = [
  "pending",
  "approved",
  "denied",
] as const;

export const deviceAuthRequests = sqliteTable("device_auth_requests", {
  // High-entropy and secret: knowing it is what lets a poller collect the token.
  id: text("id").primaryKey(),
  userCode: text("user_code").notNull(),
  label: text("label"),
  status: text("status", { enum: DEVICE_REQUEST_STATUSES })
    .notNull()
    .default("pending"),
  // Plaintext CLI token, held only until the next poll delivers it and deletes the row.
  token: text("token"),
  createdAt: createdAt(),
});

export const appState = sqliteTable("app_state", {
  key: text("key").primaryKey(),
  value: text("value").notNull(),
});
