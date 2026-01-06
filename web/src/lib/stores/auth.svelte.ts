// Mock admin authentication store
// TODO: Replace with real backend authentication when ready

import type { AuthSession } from "$lib/admin-types";

const SESSION_KEY = "admin_session";
const SESSION_DURATION_MS = 7 * 24 * 60 * 60 * 1000; // 7 days

// Mock credentials (replace with backend auth)
const MOCK_USERNAME = "admin";
const MOCK_PASSWORD = "password";

class AuthStore {
  private session = $state<AuthSession | null>(null);
  private initialized = $state(false);

  constructor() {
    // Initialize from localStorage when the store is created
    if (typeof window !== "undefined") {
      this.loadSession();
    }
  }

  get isAuthenticated(): boolean {
    if (!this.session) return false;
    
    const expiresAt = new Date(this.session.expiresAt);
    const now = new Date();
    
    if (now > expiresAt) {
      this.logout();
      return false;
    }
    
    return true;
  }

  get isInitialized(): boolean {
    return this.initialized;
  }

  private loadSession() {
    try {
      const stored = localStorage.getItem(SESSION_KEY);
      if (stored) {
        const session = JSON.parse(stored) as AuthSession;
        this.session = session;
      }
    } catch (error) {
      console.error("Failed to load session:", error);
    } finally {
      this.initialized = true;
    }
  }

  private saveSession() {
    if (this.session) {
      localStorage.setItem(SESSION_KEY, JSON.stringify(this.session));
    } else {
      localStorage.removeItem(SESSION_KEY);
    }
  }

  async login(username: string, password: string): Promise<boolean> {
    // TODO: Replace with real API call to /admin/api/login
    await new Promise((resolve) => setTimeout(resolve, 500)); // Simulate network delay

    if (username === MOCK_USERNAME && password === MOCK_PASSWORD) {
      const now = new Date();
      const expiresAt = new Date(now.getTime() + SESSION_DURATION_MS);

      this.session = {
        token: `mock-token-${Date.now()}`,
        expiresAt: expiresAt.toISOString(),
      };

      this.saveSession();
      return true;
    }

    return false;
  }

  logout() {
    this.session = null;
    this.saveSession();
  }
}

export const authStore = new AuthStore();
