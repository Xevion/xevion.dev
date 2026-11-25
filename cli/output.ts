import ansis from "ansis";
import {
  blockId,
  directText,
  formatBlockPath,
  outline,
  type DocNode,
} from "#lib/tiptap/ops.ts";
import { CliError } from "./errors.ts";
import type {
  ApiAdminProject,
  ApiSiteSettings,
  ApiTag,
  ApiTagWithCount,
  ProjectStatus,
} from "./types.ts";

// Status, progress and confirmation lines are diagnostics: they go to stderr so
// stdout carries only data (tables in human mode, JSON under --json).

/** Write a line to stderr without Bun's red console.error styling. */
export function stderr(line: string): void {
  process.stderr.write(`${line}\n`);
}

export function success(msg: string): void {
  stderr(`${ansis.green("✓")} ${msg}`);
}

export function error(msg: string): void {
  stderr(`${ansis.red("✗")} ${msg}`);
}

export function info(msg: string): void {
  stderr(`${ansis.blue("→")} ${msg}`);
}

/** Pretty-print a value as JSON to stdout, the machine-readable data channel. */
export function printJson(value: unknown): void {
  console.log(JSON.stringify(value, null, 2));
}

/** Render a failure for humans: message, per-field errors, then the help line. */
export function printCliError(err: CliError): void {
  error(err.message);
  for (const [field, message] of Object.entries(err.fieldErrors ?? {})) {
    stderr(`  ${ansis.dim(`${field}:`)} ${message}`);
  }
  if (err.help) stderr(`  ${ansis.cyan("help:")} ${err.help}`);
}

/** Paint `text`, then pad to a visible width; the pad counts unpainted characters. */
function cell(
  text: string,
  width: number,
  paint: (s: string) => string,
): string {
  const pad = Math.max(0, width - charCount(text));
  return paint(text) + " ".repeat(pad);
}

function charCount(s: string): number {
  return Array.from(s).length;
}

const plain = (s: string) => s;

function columnWidth(cells: Iterable<string>, min: number): number {
  let max = min;
  for (const c of cells) max = Math.max(max, charCount(c));
  return max;
}

export function formatStatus(status: ProjectStatus): string {
  switch (status) {
    case "active":
      return ansis.green(status);
    case "maintained":
      return ansis.blue(status);
    case "archived":
      return ansis.yellow(status);
  }
}

function field(label: string, value: string): void {
  console.log(`  ${ansis.dim(`${label}:`)} ${value}`);
}

export function printProject(project: ApiAdminProject): void {
  console.log(ansis.bold(project.name));
  field("ID", project.id);
  field("Slug", project.slug);
  field("Status", formatStatus(project.status));
  if (project.hidden || project.private) {
    const flags = [
      ...(project.hidden ? ["hidden"] : []),
      ...(project.private ? ["private"] : []),
    ];
    field("Flags", flags.join(", "));
  }
  field("Description", project.shortDescription);
  if (project.githubRepo) field("GitHub", project.githubRepo);
  if (project.demoUrl) field("Demo", project.demoUrl);
  if (project.tags.length > 0) {
    field("Tags", project.tags.map((t) => t.slug).join(", "));
  }
  field("Last Activity", project.lastActivity);
}

export function printProjectsTable(projects: ApiAdminProject[]): void {
  if (projects.length === 0) {
    info("No projects found");
    return;
  }
  const header = (s: string) => ansis.bold.underline(s);
  const nameWidth = columnWidth(
    projects.map((p) => p.name),
    4,
  );
  const slugWidth = columnWidth(
    projects.map((p) => p.slug),
    4,
  );

  console.log(
    [
      cell("NAME", nameWidth, header),
      cell("SLUG", slugWidth, header),
      cell("STATUS", 10, header),
      header("TAGS"),
    ].join("  "),
  );
  for (const p of projects) {
    const tags =
      p.tags.length === 0
        ? ansis.dim("-")
        : p.tags.map((t) => t.slug).join(", ");
    const status = formatStatus(p.status);
    console.log(
      [
        cell(p.name, nameWidth, plain),
        cell(p.slug, slugWidth, ansis.dim),
        status + " ".repeat(Math.max(0, 10 - charCount(p.status))),
        tags,
      ].join("  "),
    );
  }
  console.log();
  info(`${projects.length} project(s)`);
}

export interface BlockRow {
  /** Stable block id, absent on content that has never been stamped. */
  id?: string;
  /** Positional path (`.3.0`), the always-available addressing handle. */
  path: string;
  /** Type name, indented two spaces per level of depth to draw the tree. */
  typeLabel: string;
  /** First line of the block's own text, already truncated. */
  preview: string;
}

const PREVIEW_MAX = 60;

function previewOf(node: DocNode): string {
  const first = directText(node).split("\n")[0] ?? "";
  const chars = Array.from(first);
  return chars.length > PREVIEW_MAX
    ? `${chars.slice(0, PREVIEW_MAX).join("")}…`
    : first;
}

/** Printable rows for a document's block outline, free of ANSI escapes. */
export function blockRows(doc: DocNode): BlockRow[] {
  return outline(doc).map(({ path, node }) => ({
    id: blockId(node),
    path: formatBlockPath(path),
    typeLabel: `${"  ".repeat(Math.max(0, path.length - 1))}${node.type}`,
    preview: previewOf(node),
  }));
}

/**
 * Print a document as an indented block outline: the stable ID (`-` until
 * stamped), the positional PATH, the depth-indented TYPE and a text PREVIEW.
 */
export function printBlocks(doc: DocNode): void {
  const rows = blockRows(doc);
  if (rows.length === 0) {
    info("No content blocks");
    return;
  }
  const header = (s: string) => ansis.bold.underline(s);
  const idCell = (r: BlockRow) => (r.id === undefined ? "-" : `#${r.id}`);
  const gutter = 2;
  const idWidth = columnWidth(rows.map(idCell), 2) + gutter;
  const pathWidth =
    columnWidth(
      rows.map((r) => r.path),
      4,
    ) + gutter;
  const typeWidth =
    columnWidth(
      rows.map((r) => r.typeLabel),
      4,
    ) + gutter;

  console.log(
    cell("ID", idWidth, header) +
      cell("PATH", pathWidth, header) +
      cell("TYPE", typeWidth, header) +
      header("PREVIEW"),
  );
  for (const row of rows) {
    console.log(
      cell(idCell(row), idWidth, ansis.dim) +
        cell(row.path, pathWidth, ansis.cyan) +
        cell(row.typeLabel, typeWidth, plain) +
        ansis.dim(row.preview),
    );
  }
  console.log();
  info(`${rows.length} block(s)`);
}

export function printTag(tag: ApiTag): void {
  console.log(ansis.bold(tag.name));
  field("ID", tag.id);
  field("Slug", tag.slug);
  if (tag.icon) field("Icon", tag.icon);
  if (tag.color) field("Color", `#${tag.color}`);
}

export function printTagsTable(tags: ApiTagWithCount[]): void {
  if (tags.length === 0) {
    info("No tags found");
    return;
  }
  const header = (s: string) => ansis.bold.underline(s);
  const nameWidth = columnWidth(
    tags.map((t) => t.name),
    4,
  );
  const slugWidth = columnWidth(
    tags.map((t) => t.slug),
    4,
  );

  console.log(
    [
      cell("NAME", nameWidth, header),
      cell("SLUG", slugWidth, header),
      cell("PROJECTS", 8, header),
      cell("ICON", 20, header),
      header("COLOR"),
    ].join("  "),
  );
  for (const t of tags) {
    console.log(
      [
        cell(t.name, nameWidth, plain),
        cell(t.slug, slugWidth, ansis.dim),
        cell(String(t.projectCount), 8, plain),
        cell(t.icon || "-", 20, ansis.dim),
        ansis.dim(t.color ? `#${t.color}` : "-"),
      ].join("  "),
    );
  }
  console.log();
  info(`${tags.length} tag(s)`);
}

export function printSettings(settings: ApiSiteSettings): void {
  console.log(ansis.bold("Site Identity"));
  field("Display Name", settings.identity.displayName);
  field("Occupation", settings.identity.occupation);
  field("Bio", settings.identity.bio);
  field("Site Title", settings.identity.siteTitle);

  if (settings.socialLinks.length > 0) {
    console.log();
    console.log(ansis.bold("Social Links"));
    for (const link of settings.socialLinks) {
      const visibility = link.visible ? "" : " (hidden)";
      console.log(
        `  ${ansis.dim(`[${link.displayOrder}]`)} ${link.label}: ${link.value}${ansis.dim(visibility)}`,
      );
    }
  }
}

/** Session info goes to stderr: it is status, not pipeable data. */
export function printSession(username: string, apiUrl: string): void {
  success(`Logged in as ${ansis.bold(username)}`);
  stderr(`  ${ansis.dim("API:")} ${apiUrl}`);
}
