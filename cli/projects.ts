import { readFileSync } from "node:fs";
import {
  nonEmpty,
  parseBool,
  parseCreateTags,
  parseHexColor,
  parseStatus,
  parseUpdateTags,
} from "./args.ts";
import { authedClient, loadConfig } from "./auth.ts";
import type { ApiClient } from "./client.ts";
import { contentCommand } from "./content.ts";
import { CliError } from "./errors.ts";
import { requireFlag, type CommandNode, type Context } from "./framework.ts";
import * as output from "./output.ts";
import type {
  ApiAdminProject,
  ApiProjectDetail,
  ApiTag,
  ProjectStatus,
} from "./types.ts";

const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

const seg = encodeURIComponent;

function clientFor(ctx: Context): ApiClient {
  return authedClient(loadConfig(ctx), ctx.api);
}

/** Resolve a project reference (slug or UUID) to the full detail record. */
function resolveProject(
  client: ApiClient,
  reference: string,
): Promise<ApiProjectDetail> {
  return client.get<ApiProjectDetail>(`/api/projects/${seg(reference)}`);
}

/** Read a terminal-cast transcript from a JSON file. */
function readCast(path: string): unknown {
  let raw: string;
  try {
    raw = readFileSync(path, "utf8");
  } catch (e) {
    throw CliError.io(path, e);
  }
  try {
    return JSON.parse(raw);
  } catch (e) {
    throw CliError.json(path, e);
  }
}

async function resolveTagId(
  client: ApiClient,
  slugOrId: string,
): Promise<string> {
  if (UUID.test(slugOrId)) return slugOrId;
  const res = await client.get<{ tag?: ApiTag } & Partial<ApiTag>>(
    `/api/tags/${seg(slugOrId)}`,
  );
  const id = res.tag?.id ?? res.id;
  if (id === undefined)
    throw CliError.decode(new Error("tag response has no id"));
  return id;
}

/** Resolve comma-separated project refs to IDs, preserving authored order. */
async function resolveRelated(
  client: ApiClient,
  related: string | undefined,
): Promise<string[]> {
  if (related === undefined) return [];
  const ids: string[] = [];
  for (const ref of related
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean)) {
    ids.push((await resolveProject(client, ref)).id);
  }
  return ids;
}

function printResult(
  ctx: Context,
  project: ApiAdminProject,
  message: string,
): void {
  if (ctx.json) {
    output.printJson(project);
  } else {
    output.success(`${message}: ${project.name}`);
    output.printProject(project);
  }
}

async function list(ctx: Context): Promise<void> {
  const projects = await clientFor(ctx).get<ApiAdminProject[]>("/api/projects");
  if (ctx.json) output.printJson(projects);
  else output.printProjectsTable(projects);
}

async function get(reference: string, ctx: Context): Promise<void> {
  const project = await resolveProject(clientFor(ctx), reference);
  if (ctx.json) output.printJson(project);
  else output.printProject(project);
}

interface CreateFlags {
  shortDesc?: string;
  slug?: string;
  status?: string;
  hidden?: boolean;
  private?: boolean;
  githubRepo?: string;
  demoUrl?: string;
  tags?: string;
  accent?: string;
  projectType?: string;
  terminalCast?: string;
  related?: string;
}

async function create(
  name: string,
  f: CreateFlags,
  ctx: Context,
): Promise<void> {
  const shortDesc = requireFlag(f.shortDesc, "short-desc", "SHORT_DESC");
  const status = parseStatus(f.status ?? "active");
  const accent = f.accent === undefined ? undefined : parseHexColor(f.accent);
  const client = clientFor(ctx);

  const tagIds: string[] = [];
  for (const slug of parseCreateTags(f.tags ?? "")) {
    tagIds.push(await resolveTagId(client, slug));
  }
  const relatedIds = await resolveRelated(client, f.related);
  const cast = nonEmpty(f.terminalCast);

  const request = {
    name,
    slug: f.slug ?? null,
    shortDescription: shortDesc,
    status,
    hidden: Boolean(f.hidden),
    githubRepo: nonEmpty(f.githubRepo) ?? null,
    demoUrl: nonEmpty(f.demoUrl) ?? null,
    tagIds,
    detailContent: null,
    projectType: nonEmpty(f.projectType) ?? null,
    private: Boolean(f.private),
    terminalCast: cast === undefined ? null : readCast(cast),
    accentColor: nonEmpty(accent) ?? null,
    relatedIds,
  };
  const project = await client.post<ApiAdminProject>("/api/projects", request);
  printResult(ctx, project, "Created project");
}

interface UpdateFlags {
  name?: string;
  slug?: string;
  shortDesc?: string;
  status?: string;
  hidden?: string;
  private?: string;
  githubRepo?: string;
  demoUrl?: string;
  tags?: string;
  accent?: string;
  projectType?: string;
  terminalCast?: string;
  related?: string;
}

/** Empty string clears, absent keeps the current value. */
function merge<T>(
  flag: string | undefined,
  current: T | null | undefined,
): string | T | null {
  if (flag === undefined) return current ?? null;
  return flag === "" ? null : flag;
}

async function update(
  reference: string,
  f: UpdateFlags,
  ctx: Context,
): Promise<void> {
  const status: ProjectStatus | undefined =
    f.status === undefined ? undefined : parseStatus(f.status);
  const hidden =
    f.hidden === undefined ? undefined : parseBool("hidden", f.hidden);
  const isPrivate =
    f.private === undefined ? undefined : parseBool("private", f.private);
  const accent = f.accent === undefined ? undefined : parseHexColor(f.accent);
  const tagOps = f.tags === undefined ? [] : parseUpdateTags(f.tags);

  const client = clientFor(ctx);
  const current = await resolveProject(client, reference);

  const tagIds = current.tags.map((t) => t.id);
  for (const { op, tag } of tagOps) {
    const id = await resolveTagId(client, tag);
    if (op === "add") {
      if (!tagIds.includes(id)) tagIds.push(id);
    } else {
      const i = tagIds.indexOf(id);
      if (i >= 0) tagIds.splice(i, 1);
    }
  }

  let relatedIds = current.related.map((r) => r.id);
  if (f.related !== undefined) {
    relatedIds =
      f.related.trim() === "" ? [] : await resolveRelated(client, f.related);
  }

  // PUT is a full replace, so unchanged fields are echoed back, including the
  // detail body (which has its own `content` commands).
  const request = {
    name: f.name ?? current.name,
    slug: f.slug ?? null,
    shortDescription: f.shortDesc ?? current.shortDescription,
    status: status ?? current.status,
    hidden: hidden ?? current.hidden,
    githubRepo: merge(f.githubRepo, current.githubRepo),
    demoUrl: merge(f.demoUrl, current.demoUrl),
    tagIds,
    detailContent: current.detailContent ?? null,
    projectType: merge(f.projectType, current.projectType),
    private: isPrivate ?? current.private,
    terminalCast:
      f.terminalCast === undefined
        ? (current.terminalCast ?? null)
        : f.terminalCast === ""
          ? null
          : readCast(f.terminalCast),
    accentColor: merge(accent, current.accentColor),
    relatedIds,
  };
  const project = await client.put<ApiAdminProject>(
    `/api/projects/${seg(current.id)}`,
    request,
  );
  printResult(ctx, project, "Updated project");
}

async function remove(reference: string, ctx: Context): Promise<void> {
  const client = clientFor(ctx);
  const project = await resolveProject(client, reference);
  const deleted = await client.delete<ApiAdminProject>(
    `/api/projects/${seg(project.id)}`,
  );
  if (ctx.json) output.printJson(deleted);
  else output.success(`Deleted project: ${deleted?.name ?? project.name}`);
}

async function sync(
  reference: string | undefined,
  all: boolean,
  ctx: Context,
): Promise<void> {
  const client = clientFor(ctx);
  if (all) {
    const result = await client.post<{ enqueued: number }>("/api/github/sync");
    if (ctx.json) output.printJson(result);
    else output.success(`Enqueued ${result.enqueued} project(s) for sync`);
    return;
  }
  if (reference === undefined) {
    throw CliError.invalid("provide a project ref or --all");
  }
  const result = await client.post<ApiAdminProject | { enqueued?: number }>(
    `/api/projects/${seg(reference)}/sync`,
  );
  if (ctx.json) {
    output.printJson(result);
  } else if (result && "name" in result) {
    output.success(`Synced project: ${result.name}`);
    output.printProject(result);
  } else {
    output.success(`Enqueued ${reference} for sync`);
  }
}

export const projectsCommand: CommandNode = {
  name: "projects",
  description: "Project management",
  children: [
    {
      name: "list",
      description: "List all projects",
      run: (_p, ctx) => list(ctx),
    },
    {
      name: "get",
      description: "Get project details by slug or UUID",
      parameters: ["<ref>"],
      run: ({ _ }, ctx) => get(String(_.ref), ctx),
    },
    {
      name: "create",
      description: "Create a new project",
      parameters: ["<name>"],
      flags: {
        shortDesc: {
          type: String,
          alias: "s",
          description: "Short description (required)",
          placeholder: "<text>",
        },
        slug: {
          type: String,
          description: "URL slug (auto-generated from name if omitted)",
          placeholder: "<slug>",
        },
        status: {
          type: String,
          description:
            "Project status: active | maintained | archived [default: active]",
          placeholder: "<status>",
        },
        hidden: {
          type: Boolean,
          description: "Hide from public listings (overall visibility)",
        },
        private: {
          type: Boolean,
          description:
            "Mark the source as private: hides repo links, keeps syncing",
        },
        githubRepo: {
          type: String,
          description: 'GitHub repository (e.g., "Xevion/xevion.dev")',
          placeholder: "<repo>",
        },
        demoUrl: {
          type: String,
          description: "Demo URL",
          placeholder: "<url>",
        },
        tags: {
          type: String,
          alias: "t",
          description: "Tags to add (comma-separated, + prefix optional)",
          placeholder: "<tags>",
        },
        accent: {
          type: String,
          description: 'Accent color hex, e.g. "6366F1" or "#6366F1"',
          placeholder: "<hex>",
        },
        projectType: {
          type: String,
          description: 'Primary type label, e.g. "Web App"',
          placeholder: "<type>",
        },
        terminalCast: {
          type: String,
          description:
            "Path to a JSON file holding the terminal-cast transcript",
          placeholder: "<file>",
        },
        related: {
          type: String,
          description: "Related project slugs/ids, comma-separated",
          placeholder: "<refs>",
        },
      },
      run: ({ _, flags }, ctx) => create(String(_.name), flags, ctx),
    },
    {
      name: "update",
      description: "Update an existing project",
      parameters: ["<ref>"],
      flags: {
        name: {
          type: String,
          alias: "n",
          description: "Project name",
          placeholder: "<name>",
        },
        slug: { type: String, description: "URL slug", placeholder: "<slug>" },
        shortDesc: {
          type: String,
          alias: "s",
          description: "Short description",
          placeholder: "<text>",
        },
        status: {
          type: String,
          description: "Project status: active | maintained | archived",
          placeholder: "<status>",
        },
        hidden: {
          type: String,
          description: "Hide from public listings (true/false)",
          placeholder: "<bool>",
        },
        private: {
          type: String,
          description:
            "Mark the source as private (true/false): hides repo links, keeps syncing",
          placeholder: "<bool>",
        },
        githubRepo: {
          type: String,
          description: 'GitHub repository (use "" to clear)',
          placeholder: "<repo>",
        },
        demoUrl: {
          type: String,
          description: 'Demo URL (use "" to clear)',
          placeholder: "<url>",
        },
        tags: {
          type: String,
          alias: "t",
          description:
            "Tag changes: +tag to add, -tag to remove (comma-separated)",
          placeholder: "<tags>",
        },
        accent: {
          type: String,
          description:
            'Accent color hex, e.g. "6366F1" or "#6366F1" (use "" to clear)',
          placeholder: "<hex>",
        },
        projectType: {
          type: String,
          description: 'Primary type label, e.g. "Web App" (use "" to clear)',
          placeholder: "<type>",
        },
        terminalCast: {
          type: String,
          description:
            'Path to a JSON file with the terminal-cast transcript ("" to clear)',
          placeholder: "<file>",
        },
        related: {
          type: String,
          description:
            'Related project slugs/ids, comma-separated; full replace ("" to clear)',
          placeholder: "<refs>",
        },
      },
      run: ({ _, flags }, ctx) => update(String(_.ref), flags, ctx),
    },
    {
      name: "delete",
      description: "Delete a project",
      parameters: ["<ref>"],
      run: ({ _ }, ctx) => remove(String(_.ref), ctx),
    },
    {
      name: "sync",
      description: "Trigger an immediate GitHub activity sync",
      parameters: ["[ref]"],
      flags: {
        all: {
          type: Boolean,
          description: "Sync every project that has a GitHub repository",
        },
      },
      run: ({ _, flags }, ctx) => {
        const reference = typeof _.ref === "string" ? _.ref : undefined;
        if (flags.all && reference !== undefined) {
          throw CliError.invalid(
            "the argument '--all' cannot be used with '[ref]'",
          );
        }
        return sync(reference, Boolean(flags.all), ctx);
      },
    },
    contentCommand,
  ],
};
