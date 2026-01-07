// Admin-specific TypeScript types

export type ProjectStatus = "active" | "maintained" | "archived" | "hidden";

export interface AdminTag {
  id: string;
  slug: string;
  name: string;
  icon?: string;
  color?: string;
  createdAt?: string;
}

export interface AdminTagWithCount extends AdminTag {
  projectCount: number;
}

export interface AdminProject {
  id: string;
  slug: string;
  name: string;
  shortDescription: string;
  description: string;
  status: ProjectStatus;
  links: Array<{ url: string; title?: string }>;
  tags: AdminTag[];
  githubRepo?: string | null;
  demoUrl?: string | null;
  createdAt: string;
  updatedAt: string;
  lastGithubActivity?: string | null;
}

export interface CreateProjectData {
  name: string;
  slug?: string;
  shortDescription: string;
  description: string;
  status: ProjectStatus;
  githubRepo?: string;
  demoUrl?: string;
  tagIds: string[];
}

export interface UpdateProjectData extends CreateProjectData {
  id: string;
}

export interface CreateTagData {
  name: string;
  slug?: string;
  color?: string;
}

export interface UpdateTagData extends CreateTagData {
  id: string;
}

export type EventLevel = "info" | "warning" | "error";

export interface AdminEvent {
  id: string;
  timestamp: string; // ISO 8601
  level: EventLevel;
  target: string; // e.g., 'project.created', 'github.sync', 'tag.deleted'
  message: string;
  metadata?: Record<string, unknown>;
}

export interface AdminStats {
  totalProjects: number;
  projectsByStatus: Record<ProjectStatus, number>;
  totalTags: number;
}

export interface AuthSession {
  token: string;
  expiresAt: string; // ISO 8601
}

export type SocialPlatform =
  | "github"
  | "linkedin"
  | "discord"
  | "email"
  | "pgp";

export interface SocialLink {
  id: string;
  platform: SocialPlatform;
  label: string;
  value: string; // URL, username, or email address
  visible: boolean;
}

export interface SiteIdentity {
  displayName: string;
  occupation: string;
  bio: string;
  siteTitle: string;
}

export interface AdminPreferences {
  sessionTimeoutMinutes: number;
  eventsRetentionDays: number;
  dashboardDefaultTab: "overview" | "events";
}

export interface SiteSettings {
  identity: SiteIdentity;
  socialLinks: SocialLink[];
  adminPreferences: AdminPreferences;
}
