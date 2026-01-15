import { telemetry } from "$lib/telemetry";

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

  async login(username: string, password: string): Promise<boolean> {
    try {
      const response = await fetch("/api/login", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ username, password }),
        credentials: "include",
      });

      if (response.ok) {
        const data = await response.json();
        this.isAuthenticated = true;
        this.username = data.username;
        sessionStorage.setItem(AuthStore.STORAGE_KEY, "true");
        telemetry.identifyAdmin(data.username);
        return true;
      }

      return false;
    } catch (error) {
      console.error("Login error:", error);
      return false;
    }
  }

  async logout(): Promise<void> {
    try {
      await fetch("/api/logout", {
        method: "POST",
        credentials: "include",
      });
    } catch (error) {
      console.error("Logout error:", error);
    } finally {
      this.isAuthenticated = false;
      this.username = null;
      sessionStorage.removeItem(AuthStore.STORAGE_KEY);
      telemetry.reset();
    }
  }

  async checkSession(): Promise<boolean> {
    try {
      const response = await fetch("/api/session", {
        credentials: "include",
      });

      if (response.ok) {
        const session = await response.json();
        this.isAuthenticated = true;
        this.username = session.username;
        return true;
      }
    } catch (error) {
      console.error("Session check error:", error);
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
