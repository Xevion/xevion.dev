import { getLogger } from "@logtape/logtape";
import { telemetry } from "#lib/telemetry/index.ts";
import { apiRequest } from "#lib/api-client.ts";
import type { ApiError } from "#lib/errors.ts";

const logger = getLogger(["admin", "auth"]);

interface SessionInfo {
  username: string;
}

class AuthStore {
  private static STORAGE_KEY = "admin_session_active";

  isAuthenticated = $state(false);
  username = $state<string | null>(null);

  init() {
    if (typeof window === "undefined") return;

    const stored = sessionStorage.getItem(AuthStore.STORAGE_KEY);
    if (stored === "true") {
      this.isAuthenticated = true;
    }
  }

  /** Resolves to null on success, or the failure so the caller can tell bad credentials from rate limiting. */
  async login(username: string, password: string): Promise<ApiError | null> {
    const result = await apiRequest<SessionInfo>(
      "/api/login",
      "POST",
      { username, password },
      { redirectOnUnauthorized: false },
    );
    if (result.isErr()) return result.error;

    this.setSession(result.value.username);
    telemetry.identifyAdmin(result.value.username);
    return null;
  }

  async logout(): Promise<void> {
    const result = await apiRequest<null>("/api/logout", "POST", undefined, {
      redirectOnUnauthorized: false,
    });
    if (result.isErr()) {
      logger.error("Logout failed", { error: result.error });
    }
    this.isAuthenticated = false;
    this.username = null;
    sessionStorage.removeItem(AuthStore.STORAGE_KEY);
    telemetry.reset();
  }

  async checkSession(): Promise<boolean> {
    const result = await apiRequest<SessionInfo>(
      "/api/session",
      "GET",
      undefined,
      { redirectOnUnauthorized: false },
    );
    if (result.isOk()) {
      this.isAuthenticated = true;
      this.username = result.value.username;
      return true;
    }
    this.isAuthenticated = false;
    this.username = null;
    return false;
  }

  setSession(username: string): void {
    this.isAuthenticated = true;
    this.username = username;
    sessionStorage.setItem(AuthStore.STORAGE_KEY, "true");
  }
}

export const authStore = new AuthStore();
