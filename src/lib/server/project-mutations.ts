import { type } from "arktype";
import { waitUntil } from "cloudflare:workers";
import type { ApiAdminProject } from "#lib/bindings/index.ts";
import { getAppLogger } from "#lib/logger.ts";
import { apiError } from "./api.ts";
import { prepareContent } from "./content.ts";
import { isUniqueViolation, type Db } from "./db/index.ts";
import { fetchRepo, normalizeGithubRepo } from "./github.ts";
import { deleteProjectMediaObjects } from "./media.ts";
import { scheduleOgDelete, scheduleOgImage } from "./og.ts";
import { getAdminProject } from "./project-views.ts";
import {
  deleteProject,
  insertProject,
  replaceProject,
  type ProjectRow,
  type ProjectWrite,
} from "./projects.ts";
import { cleanRelatedIds, existingProjectIds } from "./relations.ts";
import { isUuid, normalizeHexColor, slugify } from "./slug.ts";
import { existingTagIds } from "./tags.ts";

const logger = getAppLogger("projects");

export const ProjectInputSchema = type({
  name: "string",
  "slug?": "string | null",
  shortDescription: "string",
  status: "'active' | 'maintained' | 'archived'",
  "hidden?": "boolean",
  "private?": "boolean",
  "githubRepo?": "string | null",
  "demoUrl?": "string | null",
  "tagIds?": "string[]",
  "relatedIds?": "string[]",
  "detailContent?": "unknown",
  "projectType?": "string | null",
  "accentColor?": "string | null",
  "terminalCast?": type({
    prompt: "string",
    lines: type({
      t: "'cmd' | 'out' | 'err' | 'muted'",
      text: "string",
    }).array(),
  }).or("null"),
});

type ProjectInput = typeof ProjectInputSchema.infer;

const SLUG_CONFLICT = "A project with this slug already exists";

function fail(fields: Record<string, string>): never {
  apiError(400, "VALIDATION_ERROR", Object.values(fields)[0]!, fields);
}

const blankToNull = (value: string | null | undefined): string | null =>
  value?.trim() || null;

function isHttpUrl(value: string): boolean {
  try {
    const { protocol } = new URL(value);
    return protocol === "http:" || protocol === "https:";
  } catch {
    return false;
  }
}

interface Validated {
  name: string;
  slug: string;
  shortDescription: string;
  githubRepo: string | null;
  demoUrl: string | null;
  projectType: string | null;
  accentColor: string | null;
  tagIds: string[];
  relatedIds: string[];
}

function validate(input: ProjectInput, projectId?: string): Validated {
  const errors: Record<string, string> = {};

  const name = input.name.trim();
  if (!name) errors.name = "Project name cannot be empty";

  const shortDescription = input.shortDescription.trim();
  if (!shortDescription)
    errors.shortDescription = "Project short description cannot be empty";

  const slug = slugify(blankToNull(input.slug) ?? name);
  if (name && !slug)
    errors.slug = "Slug must contain at least one letter or digit";

  const tagIds = [...new Set(input.tagIds ?? [])];
  if (!tagIds.every(isUuid)) errors.tagIds = "Invalid tag UUID format";

  const relatedIds = cleanRelatedIds(projectId ?? "", input.relatedIds ?? []);
  if (!relatedIds.every(isUuid))
    errors.relatedIds = "Invalid project UUID format";

  const rawRepo = blankToNull(input.githubRepo);
  const githubRepo = rawRepo ? normalizeGithubRepo(rawRepo) : null;
  if (rawRepo && !githubRepo)
    errors.githubRepo =
      "Enter a GitHub repository as owner/repo or a GitHub URL";

  const demoUrl = blankToNull(input.demoUrl);
  if (demoUrl && !isHttpUrl(demoUrl))
    errors.demoUrl = "Demo URL must start with http:// or https://";

  const rawAccent = blankToNull(input.accentColor);
  const accentColor = rawAccent ? normalizeHexColor(rawAccent) : null;
  if (rawAccent && !accentColor)
    errors.accentColor = "Accent must be a 6-digit hex color (e.g. '6366f1')";

  if (Object.keys(errors).length > 0) fail(errors);

  return {
    name,
    slug,
    shortDescription,
    githubRepo,
    demoUrl,
    projectType: blankToNull(input.projectType),
    accentColor,
    tagIds,
    relatedIds,
  };
}

async function checkReferences(db: Db, v: Validated): Promise<void> {
  const errors: Record<string, string> = {};
  const [knownTags, knownProjects] = await Promise.all([
    existingTagIds(db, v.tagIds),
    existingProjectIds(db, v.relatedIds),
  ]);
  if (v.tagIds.some((id) => !knownTags.has(id)))
    errors.tagIds = "One or more tags do not exist";
  if (v.relatedIds.some((id) => !knownProjects.has(id)))
    errors.relatedIds = "One or more related projects do not exist";
  if (Object.keys(errors).length > 0) fail(errors);
}

/** A definitive not-found rejects the save; an inconclusive lookup never blocks it. */
async function resolveRepo(
  repo: string,
): Promise<{ repo: string; id: number | null }> {
  const lookup = await fetchRepo(repo);
  if (lookup.kind === "found") return { repo: lookup.fullName, id: lookup.id };
  if (lookup.kind === "not-found")
    fail({ githubRepo: "GitHub repository not found or inaccessible" });
  logger.warn("github repo lookup failed, saving unresolved", {
    repo,
    message: lookup.message,
  });
  return { repo, id: null };
}

async function resolveContent(doc: unknown) {
  if (doc == null) {
    return {
      detailContent: null,
      renderedHtml: null,
      toc: null,
    };
  }
  return prepareContent(doc);
}

async function buildWrite(
  v: Validated,
  input: ProjectInput,
  github: { repo: string | null; id: number | null },
): Promise<ProjectWrite> {
  const content = await resolveContent(input.detailContent);
  return {
    slug: v.slug,
    name: v.name,
    shortDescription: v.shortDescription,
    status: input.status,
    hidden: input.hidden ?? false,
    private: input.private ?? false,
    githubRepo: github.repo,
    githubRepoId: github.id,
    demoUrl: v.demoUrl,
    projectType: v.projectType,
    accentColor: v.accentColor,
    terminalCast: input.terminalCast ?? null,
    detailContent: content.detailContent,
    renderedHtml: content.renderedHtml,
    toc: content.toc,
  };
}

export async function createProject(
  db: Db,
  input: ProjectInput,
): Promise<ApiAdminProject> {
  const v = validate(input);
  await checkReferences(db, v);
  const github = v.githubRepo
    ? await resolveRepo(v.githubRepo)
    : { repo: null, id: null };
  const write = await buildWrite(v, input, github);

  let id: string;
  try {
    id = await insertProject(db, write, v.tagIds, v.relatedIds);
  } catch (err) {
    if (isUniqueViolation(err)) apiError(409, "CONFLICT", SLUG_CONFLICT);
    throw err;
  }
  logger.info("project created", { projectId: id, slug: write.slug });
  scheduleOgImage(db, { type: "project", id });
  return (await getAdminProject(db, id))!;
}

export async function updateProject(
  db: Db,
  existing: ProjectRow,
  input: ProjectInput,
): Promise<ApiAdminProject> {
  const v = validate(input, existing.id);
  await checkReferences(db, v);

  const repoChanged = v.githubRepo !== existing.githubRepo;
  let github = { repo: v.githubRepo, id: existing.githubRepoId };
  if (repoChanged) {
    github = v.githubRepo
      ? await resolveRepo(v.githubRepo)
      : { repo: null, id: null };
  }
  const write = await buildWrite(v, input, github);
  // An omitted body keeps the stored one, so a metadata edit cannot clobber a concurrent content save.
  const { detailContent, renderedHtml, toc, ...metadata } = write;

  try {
    await replaceProject(
      db,
      existing.id,
      input.detailContent === undefined
        ? metadata
        : { ...metadata, detailContent, renderedHtml, toc },
      v.tagIds,
      v.relatedIds,
      repoChanged,
    );
  } catch (err) {
    if (isUniqueViolation(err)) apiError(409, "CONFLICT", SLUG_CONFLICT);
    throw err;
  }
  logger.info("project updated", {
    projectId: existing.id,
    slug: write.slug,
    repoChanged,
  });
  scheduleOgImage(db, { type: "project", id: existing.id });
  return (await getAdminProject(db, existing.id))!;
}

/** Returns the project as it was before deletion. */
export async function removeProject(
  db: Db,
  existing: ProjectRow,
): Promise<ApiAdminProject> {
  const snapshot = (await getAdminProject(db, existing.id))!;
  await deleteProject(db, existing.id);
  logger.info("project deleted", {
    projectId: existing.id,
    slug: existing.slug,
  });
  scheduleOgDelete(existing.id);
  waitUntil(deleteProjectMediaObjects(existing.id));
  return snapshot;
}
