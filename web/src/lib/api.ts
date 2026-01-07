import type {
  AdminProject,
  AdminTag,
  AdminTagWithCount,
  AdminEvent,
  AdminStats,
  CreateProjectData,
  UpdateProjectData,
  CreateTagData,
  UpdateTagData,
  SiteSettings,
} from "./admin-types";

// ============================================================================
// CLIENT-SIDE API FUNCTIONS
// ============================================================================

// Client-side fetch wrapper for browser requests
async function clientApiFetch<T>(
  path: string,
  init?: RequestInit,
): Promise<T> {
  const response = await fetch(path, {
    ...init,
    credentials: "same-origin", // Include cookies for auth
  });

  if (!response.ok) {
    throw new Error(`API error: ${response.status} ${response.statusText}`);
  }

  return response.json();
}

// ============================================================================
// ADMIN API FUNCTIONS
// ============================================================================

// Admin Projects API
export async function getAdminProjects(): Promise<AdminProject[]> {
  return clientApiFetch<AdminProject[]>("/api/projects");
}

export async function getAdminProject(
  id: string,
): Promise<AdminProject | null> {
  try {
    return await clientApiFetch<AdminProject>(`/api/projects/${id}`);
  } catch (error) {
    // 404 errors should return null
    if (error instanceof Error && error.message.includes("404")) {
      return null;
    }
    throw error;
  }
}

export async function createAdminProject(
  data: CreateProjectData,
): Promise<AdminProject> {
  return clientApiFetch<AdminProject>("/api/projects", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  });
}

export async function updateAdminProject(
  data: UpdateProjectData,
): Promise<AdminProject> {
  return clientApiFetch<AdminProject>(`/api/projects/${data.id}`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  });
}

export async function deleteAdminProject(id: string): Promise<AdminProject> {
  return clientApiFetch<AdminProject>(`/api/projects/${id}`, {
    method: "DELETE",
  });
}

// Admin Tags API
export async function getAdminTags(): Promise<AdminTagWithCount[]> {
  const tags = await clientApiFetch<
    Array<AdminTag & { project_count: number }>
  >("/api/tags");

  // Transform snake_case to camelCase
  return tags.map((item) => ({
    ...item,
    projectCount: item.project_count,
  }));
}

export async function createAdminTag(data: CreateTagData): Promise<AdminTag> {
  return clientApiFetch<AdminTag>("/api/tags", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  });
}

export async function updateAdminTag(data: UpdateTagData): Promise<AdminTag> {
  // Use the tag ID to construct the URL - need to get slug first
  // For now, use ID directly (may need adjustment if backend expects slug)
  return clientApiFetch<AdminTag>(`/api/tags/${data.id}`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  });
}

export async function deleteAdminTag(id: string): Promise<void> {
  // Delete by ID - may need to fetch slug first if backend expects it
  await clientApiFetch(`/api/tags/${id}`, {
    method: "DELETE",
  });
}

// Admin Events API (currently mocked - no backend implementation yet)
export async function getAdminEvents(filters?: {
  level?: string;
  target?: string;
  limit?: number;
}): Promise<AdminEvent[]> {
  // TODO: Implement when events table is added to backend
  return [];
}

// Admin Stats API
export async function getAdminStats(): Promise<AdminStats> {
  return clientApiFetch<AdminStats>("/api/stats");
}

// Settings API (currently mocked - no backend implementation yet)
export async function getSettings(): Promise<SiteSettings> {
  // TODO: Implement when settings system is added
  // For now, return default settings
  return {
    identity: {
      displayName: "Ryan Walters",
      occupation: "Full-Stack Software Engineer",
      bio: "A fanatical software engineer with expertise and passion for sound, scalable and high-performance applications. I'm always working on something new.\nSometimes innovative — sometimes crazy.",
      siteTitle: "Xevion.dev",
    },
    socialLinks: [
      {
        id: "social-1",
        platform: "github",
        label: "GitHub",
        value: "https://github.com/Xevion",
        visible: true,
      },
      {
        id: "social-2",
        platform: "linkedin",
        label: "LinkedIn",
        value: "https://linkedin.com/in/ryancwalters",
        visible: true,
      },
      {
        id: "social-3",
        platform: "discord",
        label: "Discord",
        value: "xevion",
        visible: true,
      },
    ],
    adminPreferences: {
      sessionTimeoutMinutes: 60,
      eventsRetentionDays: 30,
      dashboardDefaultTab: "overview",
    },
  };
}

export async function updateSettings(
  settings: SiteSettings,
): Promise<SiteSettings> {
  // TODO: Implement when settings system is added
  return settings;
}
