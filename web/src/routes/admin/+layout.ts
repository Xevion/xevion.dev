import { browser } from "$app/environment";
import { goto } from "$app/navigation";
import { authStore } from "$lib/stores/auth.svelte";

export const ssr = false; // Admin is client-side only

export async function load({ url }) {
  if (browser) {
    // Wait for auth store to initialize
    while (!authStore.isInitialized) {
      await new Promise((resolve) => setTimeout(resolve, 10));
    }

    // Allow access to login page without authentication
    if (url.pathname === "/admin/login") {
      // If already authenticated, redirect to dashboard
      if (authStore.isAuthenticated) {
        goto("/admin");
      }
      return {};
    }

    // Require authentication for all other admin pages
    if (!authStore.isAuthenticated) {
      goto("/admin/login");
      return {};
    }
  }

  return {};
}
