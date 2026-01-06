<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import Sidebar from "$lib/components/admin/Sidebar.svelte";
  import AppWrapper from "$lib/components/AppWrapper.svelte";
  import { authStore } from "$lib/stores/auth.svelte";
  import { getAdminStats } from "$lib/api";
  import type { AdminStats } from "$lib/admin-types";

  let { children, data } = $props();

  let stats = $state<AdminStats | null>(null);
  let loading = $state(true);

  const pathname = $derived($page.url.pathname as string);
  const isLoginPage = $derived(pathname === "/admin/login");

  // Load stats for sidebar badges
  async function loadStats() {
    if (isLoginPage || !authStore.isAuthenticated) return;
    
    try {
      stats = await getAdminStats();
    } catch (error) {
      console.error("Failed to load stats:", error);
    } finally {
      loading = false;
    }
  }

  // Sync authStore with server session on mount
  $effect(() => {
    if (data?.session?.authenticated && data.session.username && !authStore.isAuthenticated) {
      authStore.setSession(data.session.username);
    }
  });

  // Load stats when component mounts or when authentication changes
  $effect(() => {
    if (authStore.isAuthenticated && !isLoginPage) {
      loadStats();
    }
  });

  function handleLogout() {
    authStore.logout();
    goto("/admin/login");
  }
</script>

{#if isLoginPage}
  <!-- Login page has no sidebar -->
  {@render children()}
{:else}
  <!-- Admin layout with sidebar and dots shader -->
  <AppWrapper bgColor="bg-admin-bg">
    <Sidebar
      projectCount={stats?.totalProjects ?? 0}
      tagCount={stats?.totalTags ?? 0}
      onlogout={handleLogout}
    />

    <!-- Main content area -->
    <main class="lg:pl-64">
      <div class="px-4 py-8 sm:px-6 lg:px-8">
        {@render children()}
      </div>
    </main>
  </AppWrapper>
{/if}
