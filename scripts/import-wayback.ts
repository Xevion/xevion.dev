// Rebuilds the public catalogue from Wayback Machine snapshots of the old site.
//
//   bun scripts/import-wayback.ts --cache .wayback --out wayback.sql
//   bunx wrangler d1 execute xevion-dev --remote --file wayback.sql
//
// The old SvelteKit pages inlined their load data, so the homepage snapshot carries every public
// project with its tags and media, and each detail snapshot carries the raw TipTap document and
// related projects. List fields come from the homepage (the newest capture); content comes from
// the newest detail capture. Hidden projects were never served anonymously, so they cannot be
// recovered. Media rows point at the R2 objects, which survived because R2 was never on Railway.
import { existsSync, mkdirSync } from "node:fs";
import { parseArgs } from "node:util";
import vm from "node:vm";
import { SqlScript } from "./sql.ts";

const { values: args } = parseArgs({
  options: {
    cache: { type: "string", default: ".wayback" },
    out: { type: "string", default: "wayback.sql" },
    at: { type: "string", default: "20261128" },
  },
});
const cache = args.cache;
mkdirSync(cache, { recursive: true });

interface ArchivedVariant {
  url: string;
  width?: number;
  height?: number;
  mime?: string;
  duration?: number;
}

interface ArchivedMedia {
  id: string;
  displayOrder: number;
  mediaType: "image" | "video";
  variants: Record<string, ArchivedVariant | null | undefined>;
  blurhash?: string | null;
}

interface ArchivedTag {
  id: string;
  slug: string;
  name: string;
  icon: string | null;
  color: string | null;
}

interface ArchivedProject {
  id: string;
  slug: string;
  name: string;
  shortDescription: string;
  status: string;
  private: boolean;
  githubRepo: string | null;
  demoUrl: string | null;
  projectType: string | null;
  accentColor: string | null;
  createdAt: string;
  lastActivity: string | null;
  githubSyncedAt: string | null;
  tags: ArchivedTag[];
  media: ArchivedMedia[];
}

interface ArchivedDetail {
  detailContent: unknown;
  updatedAt: string;
  related: { id: string }[];
}

interface PageNode {
  data?: Record<string, unknown>;
}

const ORIGIN = "https://xevion.dev";
const MEDIA_PREFIX = "https://media.xevion.dev/";

/** Fetches the capture nearest to `--at`, caching it; null when only an error page was archived. */
async function snapshot(path: string, file: string): Promise<string | null> {
  const cached = `${cache}/${file}`;
  const missing = `${cached}.missing`;
  if (existsSync(cached)) return Bun.file(cached).text();
  if (existsSync(missing)) return null;
  for (let attempt = 1; attempt <= 3; attempt++) {
    const res = await fetch(
      `https://web.archive.org/web/${args.at}id_/${ORIGIN}${path}`,
    );
    const html = res.ok ? await res.text() : "";
    if (html.includes("kit.start(")) {
      await Bun.write(cached, html);
      console.error(`${path}: ${res.url.match(/web\/(\d+)/)?.[1]}`);
      return html;
    }
    if (res.status === 404) {
      await Bun.write(missing, res.url);
      console.error(`${path}: no capture of the page`);
      return null;
    }
    console.error(`${path}: attempt ${attempt} got ${res.status}`);
    await Bun.sleep(5000 * attempt);
  }
  throw new Error(`${path}: archive unavailable`);
}

/** The `data` array SvelteKit inlines into `kit.start(...)` for hydration. */
function hydration(html: string): (PageNode | null)[] {
  const start = html.indexOf("kit.start(app, element, ");
  const objStart = html.indexOf("{", start);
  const body = html.slice(objStart, html.indexOf("</script>", objStart));
  const literal = body.slice(
    0,
    body.indexOf("});", body.lastIndexOf("error:")) + 1,
  );
  // The payload is devalue output: an object literal, evaluated without access to any globals.
  const context = vm.createContext({ Date, Map, Set, URL });
  return (
    vm.runInContext(`(${literal})`, context, { timeout: 2000 }) as {
      data: (PageNode | null)[];
    }
  ).data;
}

function pageData<T>(html: string, key: string): T | undefined {
  return hydration(html).find((node) => node?.data?.[key] !== undefined)
    ?.data?.[key] as T | undefined;
}

const date = (value: string | null | undefined) =>
  value ? new Date(value) : null;

const home = await snapshot("/", "index.html");
if (!home) throw new Error("no homepage capture");
const settings = pageData<{
  identity: Record<string, string>;
  socialLinks: Record<string, string | number | boolean>[];
}>(home, "settings");
const projects = pageData<ArchivedProject[]>(home, "projects");
if (!settings || !projects) throw new Error("homepage capture has no data");

const script = new SqlScript();

const { identity, socialLinks } = settings;
script.insert(
  "site_identity",
  {
    id: 1,
    display_name: identity.displayName,
    occupation: identity.occupation,
    bio: identity.bio,
    site_title: identity.siteTitle,
    updated_at: new Date(),
  },
  "INSERT OR REPLACE",
);
for (const link of socialLinks) {
  script.insert(
    "social_links",
    {
      id: link.id,
      platform: link.platform,
      label: link.label,
      value: link.value,
      icon: link.icon,
      visible: link.visible,
      display_order: link.displayOrder,
    },
    "INSERT OR REPLACE",
  );
}

const tags = new Map<string, ArchivedTag>();
for (const project of projects) {
  for (const tag of project.tags) tags.set(tag.id, tag);
}
for (const tag of tags.values()) {
  script.insert(
    "tags",
    {
      id: tag.id,
      slug: tag.slug,
      name: tag.name,
      icon: tag.icon,
      color: tag.color,
      created_at: new Date(),
    },
    "INSERT OR REPLACE",
  );
}

/** Splits a media URL into the row's base path and the variant key. */
function mediaKey(url: string): { base: string; key: string } {
  if (!url.startsWith(MEDIA_PREFIX)) throw new Error(`foreign media ${url}`);
  const path = url.slice(MEDIA_PREFIX.length);
  const slash = path.lastIndexOf("/");
  return { base: path.slice(0, slash), key: path.slice(slash + 1) };
}

function storedMedia(media: ArchivedMedia) {
  const variants: Record<string, object> = {};
  let base: string | undefined;
  for (const [name, variant] of Object.entries(media.variants)) {
    if (!variant) continue;
    const parsed = mediaKey(variant.url);
    base ??= parsed.base;
    if (parsed.base !== base) throw new Error(`mixed media paths ${media.id}`);
    variants[name === "video" ? "original" : name] =
      name === "video"
        ? { key: parsed.key, mime: variant.mime, duration: variant.duration }
        : { key: parsed.key, width: variant.width, height: variant.height };
  }
  if (!base) throw new Error(`media ${media.id} has no variants`);
  return { base, variants };
}

// Children follow every project row, because D1 checks a relation against rows already inserted.
const children = new SqlScript();
const ids = new Set(projects.map((p) => p.id));
const summary: string[] = [];

for (const project of projects) {
  const html = await snapshot(
    `/projects/${project.slug}`,
    `p-${project.slug}.html`,
  );
  const detail = html ? pageData<ArchivedDetail>(html, "project") : undefined;
  await Bun.sleep(html ? 0 : 1000);

  const createdAt = date(project.createdAt)!;
  const lastActivity = date(project.lastActivity);
  script.insert(
    "projects",
    {
      id: project.id,
      slug: project.slug,
      name: project.name,
      short_description: project.shortDescription,
      status: project.status,
      hidden: false,
      private: project.private,
      github_repo: project.githubRepo,
      demo_url: project.demoUrl,
      project_type: project.projectType,
      accent_color: project.accentColor,
      detail_content: detail?.detailContent ?? null,
      // The old API reported created_at when there was no GitHub activity.
      last_github_activity:
        lastActivity && lastActivity.getTime() !== createdAt.getTime()
          ? lastActivity
          : null,
      github_synced_at: date(project.githubSyncedAt),
      github_error_count: 0,
      created_at: createdAt,
      updated_at: date(detail?.updatedAt) ?? createdAt,
    },
    "INSERT OR REPLACE",
  );

  for (const tag of project.tags) {
    children.insert("project_tags", { project_id: project.id, tag_id: tag.id });
  }

  for (const media of project.media) {
    const { base, variants } = storedMedia(media);
    children.insert("project_media", {
      id: media.id,
      project_id: project.id,
      display_order: media.displayOrder,
      media_type: media.mediaType,
      r2_base_path: base,
      variants,
      blurhash: media.blurhash ?? null,
      created_at: createdAt,
    });
  }

  const related = (detail?.related ?? []).filter((r) => ids.has(r.id));
  related.forEach((r, position) => {
    children.insert("project_relations", {
      project_id: project.id,
      related_project_id: r.id,
      position,
    });
  });

  summary.push(
    `${project.slug}\tcontent=${detail?.detailContent ? "yes" : "no"}\tmedia=${project.media.length}\trelated=${related.length}`,
  );
}

script.statements.push(...children.statements);
await Bun.write(args.out, script.toString());
console.log(summary.join("\n"));
console.error(`wrote ${script.statements.length} statements to ${args.out}`);
