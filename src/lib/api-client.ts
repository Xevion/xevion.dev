import { ResultAsync, ok, err, type Result } from "neverthrow";
import { browser } from "$app/env";
import { goto } from "$app/navigation";
import { resolve } from "$app/paths";
import { getLogger } from "@logtape/logtape";
import { NETWORK_ERROR, toApiError, type ApiError } from "#lib/errors.ts";

const logger = getLogger(["admin", "api"]);

// Latched so a page firing several calls on a dead session navigates once.
let redirecting = false;

function handleLostSession(): void {
  if (!browser || redirecting) return;
  if (window.location.pathname.startsWith("/admin/login")) return;
  redirecting = true;

  const target = `${window.location.pathname}${window.location.search}`;
  const next = target === "/admin" ? "" : `&next=${encodeURIComponent(target)}`;
  void goto(`${resolve("/admin/login")}?reason=expired${next}`, {
    replaceState: true,
    refreshAll: true,
  }).finally(() => {
    redirecting = false;
  });
}

async function toResult<T>(res: Response): Promise<Result<T, ApiError>> {
  const json: unknown = await res.json().catch(() => null);
  if (!res.ok) return err(toApiError(res.status, json));
  if (!json || typeof json !== "object" || !("data" in json)) {
    return err({
      status: res.status,
      code: "INVALID_RESPONSE",
      message: "Malformed response body",
    });
  }
  return ok((json as { data: T }).data);
}

export interface RequestOptions {
  /** Set false for calls where a 401 is an expected answer (login). */
  redirectOnUnauthorized?: boolean;
}

export function apiRequest<T>(
  url: string,
  method: string,
  body?: unknown,
  options: RequestOptions = {},
): ResultAsync<T, ApiError> {
  const isForm = body instanceof FormData;
  return ResultAsync.fromPromise(
    fetch(url, {
      method,
      credentials: "same-origin",
      headers: {
        accept: "application/json",
        ...(body !== undefined && !isForm
          ? { "content-type": "application/json" }
          : {}),
      },
      body:
        body === undefined ? undefined : isForm ? body : JSON.stringify(body),
    }),
    (cause): ApiError => {
      logger.error("Network failure", {
        method,
        url,
        message: cause instanceof Error ? cause.message : String(cause),
      });
      return NETWORK_ERROR;
    },
  )
    .andThen((res) =>
      ResultAsync.fromSafePromise(toResult<T>(res)).andThen((r) => r),
    )
    .mapErr((apiErr) => {
      if (apiErr.status === 401 && options.redirectOnUnauthorized !== false) {
        handleLostSession();
      }
      if (apiErr.code !== "NETWORK_ERROR") {
        logger.warn("Request failed", {
          method,
          url,
          status: apiErr.status,
          code: apiErr.code,
          message: apiErr.message,
        });
      }
      return apiErr;
    });
}
