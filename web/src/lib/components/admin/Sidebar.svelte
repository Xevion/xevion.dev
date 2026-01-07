<script lang="ts">
  import { page } from "$app/stores";
  import { cn } from "$lib/utils";
  import ThemeToggle from "$lib/components/ThemeToggle.svelte";
  import IconLayoutDashboard from "~icons/lucide/layout-dashboard";
  import IconFolder from "~icons/lucide/folder";
  import IconTags from "~icons/lucide/tags";
  import IconList from "~icons/lucide/list";
  import IconSettings from "~icons/lucide/settings";
  import IconArrowLeft from "~icons/lucide/arrow-left";
  import IconLogOut from "~icons/lucide/log-out";
  import IconMenu from "~icons/lucide/menu";
  import IconX from "~icons/lucide/x";

  interface Props {
    projectCount?: number;
    tagCount?: number;
    onlogout?: () => void;
  }

  let { projectCount = 0, tagCount = 0, onlogout }: Props = $props();

  let mobileMenuOpen = $state(false);

  interface NavItem {
    href: string;
    label: string;
    icon: import("svelte").Component;
    badge?: number;
  }

  const navItems = $derived<NavItem[]>([
    { href: "/admin", label: "Dashboard", icon: IconLayoutDashboard },
    {
      href: "/admin/projects",
      label: "Projects",
      icon: IconFolder,
      badge: projectCount,
    },
    { href: "/admin/tags", label: "Tags", icon: IconTags, badge: tagCount },
    { href: "/admin/events", label: "Events", icon: IconList },
    { href: "/admin/settings", label: "Settings", icon: IconSettings },
  ]);

  const pathname = $derived($page.url.pathname as string);

  function isActive(href: string): boolean {
    if (href === "/admin") {
      return pathname === "/admin";
    }
    return pathname.startsWith(href);
  }

  function handleLogout() {
    onlogout?.();
  }
</script>

<!-- Mobile menu button -->
<button
  class="fixed top-4 right-4 z-50 lg:hidden rounded-md bg-admin-surface p-2 text-admin-text border border-admin-border"
  onclick={() => (mobileMenuOpen = !mobileMenuOpen)}
  aria-label="Toggle menu"
>
  {#if mobileMenuOpen}
    <IconX class="w-5 h-5" />
  {:else}
    <IconMenu class="w-5 h-5" />
  {/if}
</button>

<!-- Sidebar -->
<aside
  class={cn(
    "fixed left-0 top-0 z-40 h-screen w-64 border-r border-admin-border bg-admin-bg transition-transform lg:translate-x-0",
    mobileMenuOpen ? "translate-x-0" : "-translate-x-full",
  )}
>
  <div class="flex h-full flex-col">
    <!-- Logo -->
    <div class="border-b border-admin-border px-4 py-5 flex items-center justify-between">
      <h1 class="text-base font-semibold text-admin-text">
        xevion.dev
        <span class="text-xs font-normal text-admin-text-muted ml-1.5">Admin</span>
      </h1>
      <ThemeToggle />
    </div>

    <!-- Navigation -->
    <nav class="flex-1 space-y-0.5 p-3">
      {#each navItems as item (item.href)}
        <a
          href={item.href}
          class={cn(
            "flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-all relative",
            isActive(item.href)
              ? "bg-admin-surface-hover text-admin-text before:absolute before:left-0 before:top-1 before:bottom-1 before:w-0.5 before:bg-admin-accent before:rounded-r"
              : "text-admin-text-muted hover:text-admin-text hover:bg-admin-surface-hover/50",
          )}
        >
          <item.icon class="w-4 h-4 flex-shrink-0" />
          <span class="flex-1">{item.label}</span>
          {#if item.badge}
            <span class="text-xs text-admin-text-muted">
              {item.badge}
            </span>
          {/if}
        </a>
      {/each}
    </nav>

    <!-- Bottom actions -->
    <div class="space-y-0.5 border-t border-admin-border bg-admin-surface/50 p-3">
      <a
        href="/"
        class="flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium text-admin-text-muted transition-all hover:text-admin-text hover:bg-admin-surface-hover/50"
      >
        <IconArrowLeft class="w-4 h-4" />
        <span>Back to Site</span>
      </a>
      <button
        onclick={handleLogout}
        class="flex w-full items-center gap-3 rounded-md px-3 py-2 text-sm font-medium text-admin-text-muted transition-all hover:text-admin-text hover:bg-admin-surface-hover/50"
      >
        <IconLogOut class="w-4 h-4" />
        <span>Logout</span>
      </button>
    </div>
  </div>
</aside>

<!-- Backdrop for mobile -->
{#if mobileMenuOpen}
  <div
    class="fixed inset-0 z-30 bg-black/50 lg:hidden"
    onclick={() => (mobileMenuOpen = false)}
    onkeydown={(e) => e.key === "Escape" && (mobileMenuOpen = false)}
    role="presentation"
    tabindex="-1"
  ></div>
{/if}
