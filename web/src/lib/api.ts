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
  ProjectMedia,
} from "./admin-types";
import { ApiError } from "./errors";

// Client-side fetch wrapper for browser requests
async function clientApiFetch<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(path, {
    ...init,
    credentials: "same-origin", // Include cookies for auth
  });

  if (!response.ok) {
    throw new ApiError(response.status, response.statusText);
  }

  return response.json();
}

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
    if (ApiError.isNotFound(error)) {
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
  return clientApiFetch<AdminTagWithCount[]>("/api/tags");
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

export interface TagWithProjects {
  tag: AdminTag;
  projects: AdminProject[];
}

export async function getAdminTagBySlug(
  slug: string,
): Promise<TagWithProjects | null> {
  try {
    return await clientApiFetch<TagWithProjects>(`/api/tags/${slug}`);
  } catch (error) {
    if (ApiError.isNotFound(error)) {
      return null;
    }
    throw error;
  }
}

export interface RelatedTag extends AdminTag {
  cooccurrenceCount: number;
}

export async function getRelatedTags(slug: string): Promise<RelatedTag[]> {
  return clientApiFetch<RelatedTag[]>(`/api/tags/${slug}/related`);
}

// Admin Media API
export async function uploadProjectMedia(
  projectId: string,
  file: File,
  onProgress?: (percent: number) => void,
): Promise<ProjectMedia> {
  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest();
    const formData = new FormData();
    formData.append("file", file);

    xhr.upload.onprogress = (e) => {
      if (e.lengthComputable && onProgress) {
        onProgress(Math.round((e.loaded / e.total) * 100));
      }
    };

    xhr.onload = () => {
      if (xhr.status >= 200 && xhr.status < 300) {
        try {
          const data = JSON.parse(xhr.responseText);
          resolve(data as ProjectMedia);
        } catch {
          reject(new Error("Invalid response from server"));
        }
      } else {
        reject(new Error(`Upload failed: ${xhr.statusText}`));
      }
    };

    xhr.onerror = () => reject(new Error("Network error during upload"));

    xhr.open("POST", `/api/projects/${projectId}/media`);
    xhr.withCredentials = true;
    xhr.send(formData);
  });
}

export async function deleteProjectMedia(
  projectId: string,
  mediaId: string,
): Promise<ProjectMedia> {
  return clientApiFetch<ProjectMedia>(
    `/api/projects/${projectId}/media/${mediaId}`,
    { method: "DELETE" },
  );
}

export async function reorderProjectMedia(
  projectId: string,
  mediaIds: string[],
): Promise<ProjectMedia[]> {
  return clientApiFetch<ProjectMedia[]>(
    `/api/projects/${projectId}/media/reorder`,
    {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ mediaIds }),
    },
  );
}

// Admin Events API (currently mocked - no backend implementation yet)
export async function getAdminEvents(): Promise<AdminEvent[]> {
  // TODO: Implement when events table is added to backend
  // filters parameter will be added when backend implementation is complete
  return [];
}

// Admin Stats API
export async function getAdminStats(): Promise<AdminStats> {
  return clientApiFetch<AdminStats>("/api/stats");
}

// Settings API
export async function getSettings(): Promise<SiteSettings> {
  return clientApiFetch<SiteSettings>("/api/settings");
}

export async function updateSettings(
  settings: SiteSettings,
): Promise<SiteSettings> {
  return clientApiFetch<SiteSettings>("/api/settings", {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(settings),
  });
}
