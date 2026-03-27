import type {
  ApiAdminProject,
  ApiTag,
  ApiTagWithCount,
  AdminStats,
  ApiSiteSettings,
  ApiProjectMedia,
} from "$lib/bindings";
import type {
  AdminEvent,
  CreateProjectData,
  UpdateProjectData,
  CreateTagData,
  UpdateTagData,
} from "./admin-types";
import { ApiError } from "./errors";

async function clientApiFetch<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(path, {
    ...init,
    credentials: "same-origin",
  });

  if (!response.ok) {
    throw new ApiError(response.status, response.statusText);
  }

  return response.json();
}

export async function getAdminProjects(): Promise<ApiAdminProject[]> {
  return clientApiFetch<ApiAdminProject[]>("/api/projects");
}

export async function getAdminProject(
  id: string,
): Promise<ApiAdminProject | null> {
  try {
    return await clientApiFetch<ApiAdminProject>(`/api/projects/${id}`);
  } catch (error) {
    if (ApiError.isNotFound(error)) {
      return null;
    }
    throw error;
  }
}

export async function createAdminProject(
  data: CreateProjectData,
): Promise<ApiAdminProject> {
  return clientApiFetch<ApiAdminProject>("/api/projects", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  });
}

export async function updateAdminProject(
  data: UpdateProjectData,
): Promise<ApiAdminProject> {
  return clientApiFetch<ApiAdminProject>(`/api/projects/${data.id}`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  });
}

export async function deleteAdminProject(id: string): Promise<ApiAdminProject> {
  return clientApiFetch<ApiAdminProject>(`/api/projects/${id}`, {
    method: "DELETE",
  });
}

export async function getAdminTags(): Promise<ApiTagWithCount[]> {
  return clientApiFetch<ApiTagWithCount[]>("/api/tags");
}

export async function createAdminTag(data: CreateTagData): Promise<ApiTag> {
  return clientApiFetch<ApiTag>("/api/tags", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  });
}

export async function updateAdminTag(data: UpdateTagData): Promise<ApiTag> {
  return clientApiFetch<ApiTag>(`/api/tags/${data.id}`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  });
}

export async function deleteAdminTag(id: string): Promise<void> {
  await clientApiFetch(`/api/tags/${id}`, {
    method: "DELETE",
  });
}

export interface TagWithProjects {
  tag: ApiTag;
  projects: ApiAdminProject[];
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

export interface RelatedTag extends ApiTag {
  cooccurrenceCount: number;
}

export async function getRelatedTags(slug: string): Promise<RelatedTag[]> {
  return clientApiFetch<RelatedTag[]>(`/api/tags/${slug}/related`);
}

export async function uploadProjectMedia(
  projectId: string,
  file: File,
  onProgress?: (percent: number) => void,
): Promise<ApiProjectMedia> {
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
          resolve(data as ApiProjectMedia);
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
): Promise<ApiProjectMedia> {
  return clientApiFetch<ApiProjectMedia>(
    `/api/projects/${projectId}/media/${mediaId}`,
    { method: "DELETE" },
  );
}

export async function reorderProjectMedia(
  projectId: string,
  mediaIds: string[],
): Promise<ApiProjectMedia[]> {
  return clientApiFetch<ApiProjectMedia[]>(
    `/api/projects/${projectId}/media/reorder`,
    {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ mediaIds }),
    },
  );
}

// TODO: Implement when events table is added to backend
export async function getAdminEvents(): Promise<AdminEvent[]> {
  return [];
}

export async function getAdminStats(): Promise<AdminStats> {
  return clientApiFetch<AdminStats>("/api/stats");
}

export async function getSettings(): Promise<ApiSiteSettings> {
  return clientApiFetch<ApiSiteSettings>("/api/settings");
}

export async function updateSettings(
  settings: ApiSiteSettings,
): Promise<ApiSiteSettings> {
  return clientApiFetch<ApiSiteSettings>("/api/settings", {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(settings),
  });
}
