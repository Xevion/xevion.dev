import type {
  ApiAdminProject,
  ApiEvent,
  ApiTag,
  ApiTagWithCount,
  AdminStats,
  ApiSiteSettings,
  ApiProjectMedia,
  EventLevel,
} from "$lib/bindings";
import type {
  CreateProjectData,
  UpdateProjectData,
  CreateTagData,
  UpdateTagData,
} from "./admin-types";
import { ApiError } from "./errors";
import { ok, err } from "true-myth/result";
import type { Result } from "true-myth/result";

async function clientApiFetch<T>(
  path: string,
  init?: RequestInit,
): Promise<Result<T, ApiError>> {
  try {
    const response = await fetch(path, {
      ...init,
      credentials: "same-origin",
    });

    if (!response.ok) {
      return err(await ApiError.fromResponse(response));
    }

    return ok(await response.json());
  } catch (error) {
    return err(ApiError.network(error));
  }
}

export async function getAdminProjects(): Promise<
  Result<ApiAdminProject[], ApiError>
> {
  return clientApiFetch<ApiAdminProject[]>("/api/projects");
}

export async function getAdminProject(
  id: string,
): Promise<Result<ApiAdminProject | null, ApiError>> {
  const result = await clientApiFetch<ApiAdminProject>(`/api/projects/${id}`);
  if (result.isErr) {
    return ApiError.isNotFound(result.error) ? ok(null) : err(result.error);
  }
  return ok(result.value);
}

export async function createAdminProject(
  data: CreateProjectData,
): Promise<Result<ApiAdminProject, ApiError>> {
  return clientApiFetch<ApiAdminProject>("/api/projects", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  });
}

export async function updateAdminProject(
  data: UpdateProjectData,
): Promise<Result<ApiAdminProject, ApiError>> {
  return clientApiFetch<ApiAdminProject>(`/api/projects/${data.id}`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  });
}

export async function deleteAdminProject(
  id: string,
): Promise<Result<ApiAdminProject, ApiError>> {
  return clientApiFetch<ApiAdminProject>(`/api/projects/${id}`, {
    method: "DELETE",
  });
}

export async function getAdminTags(): Promise<
  Result<ApiTagWithCount[], ApiError>
> {
  return clientApiFetch<ApiTagWithCount[]>("/api/tags");
}

export async function createAdminTag(
  data: CreateTagData,
): Promise<Result<ApiTag, ApiError>> {
  return clientApiFetch<ApiTag>("/api/tags", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  });
}

export async function updateAdminTag(
  data: UpdateTagData,
): Promise<Result<ApiTag, ApiError>> {
  return clientApiFetch<ApiTag>(`/api/tags/${data.id}`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  });
}

export async function deleteAdminTag(
  id: string,
): Promise<Result<void, ApiError>> {
  return clientApiFetch<void>(`/api/tags/${id}`, {
    method: "DELETE",
  });
}

export interface TagWithProjects {
  tag: ApiTag;
  projects: ApiAdminProject[];
}

export async function getAdminTagBySlug(
  slug: string,
): Promise<Result<TagWithProjects | null, ApiError>> {
  const result = await clientApiFetch<TagWithProjects>(`/api/tags/${slug}`);
  if (result.isErr) {
    return ApiError.isNotFound(result.error) ? ok(null) : err(result.error);
  }
  return ok(result.value);
}

export interface RelatedTag extends ApiTag {
  cooccurrenceCount: number;
}

export async function getRelatedTags(
  slug: string,
): Promise<Result<RelatedTag[], ApiError>> {
  return clientApiFetch<RelatedTag[]>(`/api/tags/${slug}/related`);
}

export async function uploadProjectMedia(
  projectId: string,
  file: File,
  onProgress?: (percent: number) => void,
): Promise<Result<ApiProjectMedia, ApiError>> {
  return new Promise((resolve) => {
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
          const data = JSON.parse(xhr.responseText) as ApiProjectMedia;
          resolve(ok(data));
        } catch {
          resolve(
            err(new ApiError(xhr.status, xhr.statusText, "Invalid response")),
          );
        }
        return;
      }

      // Non-2xx: read the JSON error body for message + field errors.
      let message: string | undefined;
      let fieldErrors: Record<string, string> | undefined;
      try {
        const body = JSON.parse(xhr.responseText);
        if (body && typeof body === "object") {
          if (typeof body.error === "string") message = body.error;
          if (
            body.fieldErrors &&
            typeof body.fieldErrors === "object" &&
            !Array.isArray(body.fieldErrors)
          ) {
            fieldErrors = body.fieldErrors as Record<string, string>;
          }
        }
      } catch {
        // Body is not JSON — fall back to statusText.
      }
      resolve(
        err(new ApiError(xhr.status, xhr.statusText, message, fieldErrors)),
      );
    };

    xhr.onerror = () => resolve(err(ApiError.network("Network error")));

    xhr.open("POST", `/api/projects/${projectId}/media`);
    xhr.withCredentials = true;
    xhr.send(formData);
  });
}

export async function deleteProjectMedia(
  projectId: string,
  mediaId: string,
): Promise<Result<ApiProjectMedia, ApiError>> {
  return clientApiFetch<ApiProjectMedia>(
    `/api/projects/${projectId}/media/${mediaId}`,
    { method: "DELETE" },
  );
}

export async function reorderProjectMedia(
  projectId: string,
  mediaIds: string[],
): Promise<Result<ApiProjectMedia[], ApiError>> {
  return clientApiFetch<ApiProjectMedia[]>(
    `/api/projects/${projectId}/media/reorder`,
    {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ mediaIds }),
    },
  );
}

export async function getAdminEvents(params?: {
  limit?: number;
  offset?: number;
  level?: EventLevel;
  entityType?: string;
  eventType?: string;
}): Promise<Result<ApiEvent[], ApiError>> {
  const search = new URLSearchParams();
  if (params?.limit != null) search.set("limit", String(params.limit));
  if (params?.offset != null) search.set("offset", String(params.offset));
  if (params?.level) search.set("level", params.level);
  if (params?.entityType) search.set("entityType", params.entityType);
  if (params?.eventType) search.set("eventType", params.eventType);
  const qs = search.toString();
  return clientApiFetch<ApiEvent[]>(`/api/events${qs ? `?${qs}` : ""}`);
}

export async function getAdminStats(): Promise<Result<AdminStats, ApiError>> {
  return clientApiFetch<AdminStats>("/api/stats");
}

export async function getSettings(): Promise<
  Result<ApiSiteSettings, ApiError>
> {
  return clientApiFetch<ApiSiteSettings>("/api/settings");
}

export async function updateSettings(
  settings: ApiSiteSettings,
): Promise<Result<ApiSiteSettings, ApiError>> {
  return clientApiFetch<ApiSiteSettings>("/api/settings", {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(settings),
  });
}
