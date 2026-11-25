import type {
  ApiAdminProject,
  ApiProject,
  ApiProjectDetail,
  ApiProjectLink,
  ApiProjectMedia,
  ApiRelatedProject,
  ApiTag,
  ProjectStatus,
  TerminalCast,
} from "#lib/bindings/index.ts";

/** The project columns every list and detail response is built from. */
export interface ProjectCore {
  id: string;
  slug: string;
  name: string;
  shortDescription: string;
  status: ProjectStatus;
  hidden: boolean;
  private: boolean;
  githubRepo: string | null;
  demoUrl: string | null;
  projectType: string | null;
  accentColor: string | null;
  lastGithubActivity: Date | null;
  githubSyncedAt: Date | null;
  githubSyncError: string | null;
  createdAt: Date;
}

export interface ProjectDetailCore extends ProjectCore {
  terminalCast: TerminalCast | null;
  detailContent: unknown;
  updatedAt: Date;
}

/** A private repo is still synced, but its source link never appears. */
export function buildLinks(
  project: Pick<ProjectCore, "githubRepo" | "demoUrl" | "private">,
): ApiProjectLink[] {
  const links: ApiProjectLink[] = [];
  if (project.githubRepo && !project.private) {
    links.push({
      url: `https://github.com/${project.githubRepo}`,
      title: "GitHub",
    });
  }
  if (project.demoUrl) links.push({ url: project.demoUrl, title: "Demo" });
  return links;
}

export function toApiProject(p: ProjectCore): ApiProject {
  return {
    id: p.id,
    slug: p.slug,
    name: p.name,
    shortDescription: p.shortDescription,
    links: buildLinks(p),
  };
}

export function toApiAdminProject(
  p: ProjectCore,
  tags: ApiTag[],
  media: ApiProjectMedia[],
): ApiAdminProject {
  const out: ApiAdminProject = {
    ...toApiProject(p),
    tags,
    media,
    status: p.status,
    hidden: p.hidden,
    createdAt: p.createdAt.toISOString(),
    lastActivity: (p.lastGithubActivity ?? p.createdAt).toISOString(),
    private: p.private,
  };
  if (p.githubRepo) out.githubRepo = p.githubRepo;
  if (p.demoUrl) out.demoUrl = p.demoUrl;
  if (p.projectType) out.projectType = p.projectType;
  if (p.accentColor) out.accentColor = p.accentColor;
  if (p.githubSyncedAt) out.githubSyncedAt = p.githubSyncedAt.toISOString();
  if (p.githubSyncError) out.githubSyncError = p.githubSyncError;
  return out;
}

export function toApiProjectDetail(
  p: ProjectDetailCore,
  tags: ApiTag[],
  media: ApiProjectMedia[],
  related: ApiRelatedProject[],
): ApiProjectDetail {
  const out: ApiProjectDetail = {
    ...toApiAdminProject(p, tags, media),
    updatedAt: p.updatedAt.toISOString(),
    related,
  };
  if (p.detailContent != null) out.detailContent = p.detailContent;
  if (p.terminalCast) out.terminalCast = p.terminalCast;
  return out;
}
