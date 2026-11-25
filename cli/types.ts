export type ProjectStatus = "active" | "maintained" | "archived";

export const PROJECT_STATUSES: readonly ProjectStatus[] = [
  "active",
  "maintained",
  "archived",
];

export interface ApiTag {
  id: string;
  slug: string;
  name: string;
  icon?: string | null;
  color?: string | null;
}

export interface ApiTagWithCount extends ApiTag {
  projectCount: number;
}

export interface ApiRelatedProject {
  id: string;
  slug: string;
  name: string;
}

/** Fields of the list/detail project responses the CLI reads. */
export interface ApiAdminProject {
  id: string;
  slug: string;
  name: string;
  shortDescription: string;
  status: ProjectStatus;
  hidden: boolean;
  private: boolean;
  githubRepo?: string | null;
  demoUrl?: string | null;
  projectType?: string | null;
  accentColor?: string | null;
  lastActivity: string;
  tags: ApiTag[];
}

export interface ApiProjectDetail extends ApiAdminProject {
  detailContent?: unknown;
  terminalCast?: unknown;
  related: ApiRelatedProject[];
}

export interface ApiSocialLink {
  id: string;
  platform: string;
  label: string;
  value: string;
  icon: string;
  visible: boolean;
  displayOrder: number;
}

export interface ApiSiteSettings {
  identity: {
    displayName: string;
    occupation: string;
    bio: string;
    siteTitle: string;
  };
  socialLinks: ApiSocialLink[];
}

export interface ApiSession {
  authenticated: boolean;
  username: string;
  sessionType: string;
  expiresAt: string;
}
