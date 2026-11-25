import type {
  ApiAdminProject,
  ApiTag,
  ApiTagWithCount,
  AdminStats,
  ApiSiteIdentity,
  ApiSiteSettings,
  ApiProjectDetail,
  ApiProjectMedia,
  ApiRelatedTag,
  ApiSession,
} from "#lib/bindings/index.ts";
import { ResultAsync, ok, err, type Result } from "neverthrow";
import type {
  CreateProjectData,
  UpdateProjectData,
  CreateTagData,
  UpdateTagData,
} from "./admin-types";
import { apiRequest } from "./api-client";
import { NETWORK_ERROR, toApiError, type ApiError } from "./errors";

export function getAdminProjects(): ResultAsync<ApiAdminProject[], ApiError> {
  return apiRequest<ApiAdminProject[]>("/api/projects", "GET");
}

export function getAdminProject(
  id: string,
): ResultAsync<ApiProjectDetail, ApiError> {
  return apiRequest<ApiProjectDetail>(`/api/projects/${id}`, "GET");
}

export function createAdminProject(
  data: CreateProjectData,
): ResultAsync<ApiAdminProject, ApiError> {
  return apiRequest<ApiAdminProject>("/api/projects", "POST", data);
}

export function updateAdminProject(
  data: UpdateProjectData,
): ResultAsync<ApiAdminProject, ApiError> {
  const { id, ...body } = data;
  return apiRequest<ApiAdminProject>(`/api/projects/${id}`, "PUT", body);
}

export function deleteAdminProject(
  id: string,
): ResultAsync<ApiAdminProject, ApiError> {
  return apiRequest<ApiAdminProject>(`/api/projects/${id}`, "DELETE");
}

export function getAdminTags(): ResultAsync<ApiTagWithCount[], ApiError> {
  return apiRequest<ApiTagWithCount[]>("/api/tags", "GET");
}

export function createAdminTag(
  data: CreateTagData,
): ResultAsync<ApiTag, ApiError> {
  return apiRequest<ApiTag>("/api/tags", "POST", data);
}

export function updateAdminTag(
  data: UpdateTagData,
): ResultAsync<ApiTag, ApiError> {
  const { id, ...body } = data;
  return apiRequest<ApiTag>(`/api/tags/${id}`, "PUT", body);
}

export function deleteAdminTag(id: string): ResultAsync<unknown, ApiError> {
  return apiRequest<unknown>(`/api/tags/${id}`, "DELETE");
}

export interface TagWithProjects {
  tag: ApiTag;
  projects: ApiAdminProject[];
}

export function getAdminTagBySlug(
  slug: string,
): ResultAsync<TagWithProjects, ApiError> {
  return apiRequest<TagWithProjects>(`/api/tags/${slug}`, "GET");
}

export function getRelatedTags(
  slug: string,
): ResultAsync<ApiRelatedTag[], ApiError> {
  return apiRequest<ApiRelatedTag[]>(`/api/tags/${slug}/related`, "GET");
}

export function getProjectMedia(
  projectId: string,
): ResultAsync<ApiProjectMedia[], ApiError> {
  return apiRequest<ApiProjectMedia[]>(
    `/api/projects/${projectId}/media`,
    "GET",
  );
}

export function uploadProjectMedia(
  projectId: string,
  file: File,
  onProgress?: (percent: number) => void,
  blurhash?: string,
): ResultAsync<ApiProjectMedia, ApiError> {
  return ResultAsync.fromSafePromise(
    new Promise<Result<ApiProjectMedia, ApiError>>((resolve) => {
      const xhr = new XMLHttpRequest();
      const formData = new FormData();
      formData.append("file", file);
      if (blurhash) formData.append("blurhash", blurhash);

      xhr.upload.onprogress = (e) => {
        if (e.lengthComputable && onProgress) {
          onProgress(Math.round((e.loaded / e.total) * 100));
        }
      };

      xhr.onload = () => {
        let json: unknown = null;
        try {
          json = JSON.parse(xhr.responseText);
        } catch {
          // not JSON
        }
        if (xhr.status >= 200 && xhr.status < 300) {
          if (json && typeof json === "object" && "data" in json) {
            resolve(ok((json as { data: ApiProjectMedia }).data));
          } else {
            resolve(
              err({
                status: xhr.status,
                code: "INVALID_RESPONSE",
                message: "Malformed response body",
              }),
            );
          }
          return;
        }
        resolve(err(toApiError(xhr.status, json)));
      };

      xhr.onerror = () => resolve(err(NETWORK_ERROR));

      xhr.open("POST", `/api/projects/${projectId}/media`);
      xhr.withCredentials = true;
      xhr.send(formData);
    }),
  ).andThen((r) => r);
}

export function deleteProjectMedia(
  projectId: string,
  mediaId: string,
): ResultAsync<unknown, ApiError> {
  return apiRequest<unknown>(
    `/api/projects/${projectId}/media/${mediaId}`,
    "DELETE",
  );
}

export function reorderProjectMedia(
  projectId: string,
  mediaIds: string[],
): ResultAsync<ApiProjectMedia[], ApiError> {
  return apiRequest<ApiProjectMedia[]>(
    `/api/projects/${projectId}/media/reorder`,
    "PUT",
    { mediaIds },
  );
}

export function getAdminStats(): ResultAsync<AdminStats, ApiError> {
  return apiRequest<AdminStats>("/api/stats", "GET");
}

export function getSettings(): ResultAsync<ApiSiteSettings, ApiError> {
  return apiRequest<ApiSiteSettings>("/api/settings", "GET");
}

export function updateSettings(
  settings: ApiSiteSettings,
): ResultAsync<ApiSiteSettings, ApiError> {
  return apiRequest<ApiSiteSettings>("/api/settings", "PUT", settings);
}

export function uploadResume(
  file: File,
): ResultAsync<ApiSiteIdentity, ApiError> {
  const formData = new FormData();
  formData.append("file", file);
  return apiRequest<ApiSiteIdentity>("/api/settings/resume", "POST", formData);
}

export function getSessions(): ResultAsync<ApiSession[], ApiError> {
  return apiRequest<ApiSession[]>("/api/sessions", "GET");
}

export function revokeSession(id: string): ResultAsync<unknown, ApiError> {
  return apiRequest<unknown>(`/api/sessions/${id}`, "DELETE");
}
