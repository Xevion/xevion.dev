<script lang="ts">
  import "@fontsource-variable/inter/wght.css";
  import "@fontsource/hanken-grotesk/900.css";
  import "@fontsource-variable/schibsted-grotesk/wght.css";
  import "overlayscrollbars/overlayscrollbars.css";
  import "../app.css";
  import { OverlayScrollbars } from "overlayscrollbars";
  import { onMount } from "svelte";
  import { themeStore } from "$lib/stores/theme.svelte";
  import { page } from "$app/stores";
  import { onNavigate } from "$app/navigation";
  import Clouds from "$lib/components/Clouds.svelte";
  import Dots from "$lib/components/Dots.svelte";
  import ThemeToggle from "$lib/components/ThemeToggle.svelte";

  let { children, data } = $props();

  // Randomly choose background component on mount (stable, doesn't change after initial load)
  let backgroundComponent = $state<"clouds" | "dots" | null>(null);

  const defaultMetadata = {
    title: "Xevion.dev",
    description:
      "The personal website of Xevion, a full-stack software developer.",
    ogImage: "/api/og/home.png",
    url: "https://xevion.dev",
  };

  const metadata = $derived(data?.metadata ?? defaultMetadata);

  // Check if current route is admin (admin has its own layout/background)
  const isAdminRoute = $derived($page.url.pathname.startsWith("/admin"));
  // Check if current route is internal (OG preview, etc.)
  const isInternalRoute = $derived($page.url.pathname.startsWith("/internal"));
  // Show global background for public pages only
  const showGlobalBackground = $derived(!isAdminRoute && !isInternalRoute);

  // Use View Transitions API for smooth page transitions (Chrome 111+, Safari 18+)
  onNavigate((navigation) => {
    // Skip transitions for same-page navigations or if API not supported
    if (
      !document.startViewTransition ||
      navigation.from?.url.pathname === navigation.to?.url.pathname
    ) {
      return;
    }

    // Skip transitions for admin routes (they have their own layout/style)
    const fromAdmin = navigation.from?.url.pathname.startsWith("/admin");
    const toAdmin = navigation.to?.url.pathname.startsWith("/admin");
    if (fromAdmin || toAdmin) {
      return;
    }

    return new Promise((resolve) => {
      document.startViewTransition(async () => {
        resolve();
        await navigation.complete;
      });
    });
  });

  onMount(() => {
    // Detect if this is a page reload (F5 or CTRL+F5) vs initial load or SPA navigation
    const navigation = performance.getEntriesByType(
      "navigation",
    )[0] as PerformanceNavigationTiming;
    const isReload = navigation?.type === "reload";

    // Randomize on reload OR if not yet set (initial load)
    // SPA navigation doesn't trigger onMount, so background stays stable
    if (isReload || backgroundComponent === null) {
      backgroundComponent = Math.random() < 0.5 ? "clouds" : "dots";
    }

    // Initialize theme store
    themeStore.init();

    // Initialize overlay scrollbars on the body element
    const osInstance = OverlayScrollbars(document.body, {
      scrollbars: {
        autoHide: "leave",
        autoHideDelay: 800,
        theme: themeStore.isDark ? "os-theme-dark" : "os-theme-light",
      },
    });

    return () => {
      osInstance?.destroy();
    };
  });
</script>

<svelte:head>
  <link rel="icon" href="/favicon.ico" sizes="32x32" />
  <link rel="icon" href="/favicon-192.png" type="image/png" sizes="192x192" />
  <link rel="apple-touch-icon" href="/apple-touch-icon-180.png" />

  <!-- Primary Meta Tags -->
  <title>{metadata.title}</title>
  <meta name="description" content={metadata.description} />

  <!-- Open Graph Meta Tags -->
  <meta property="og:type" content="website" />
  <meta property="og:url" content={metadata.url} />
  <meta property="og:title" content={metadata.title} />
  <meta property="og:description" content={metadata.description} />
  <meta property="og:image" content={metadata.ogImage} />
  <meta property="og:image:width" content="1200" />
  <meta property="og:image:height" content="630" />

  <!-- Twitter Card Meta Tags -->
  <meta name="twitter:card" content="summary_large_image" />
  <meta name="twitter:title" content={metadata.title} />
  <meta name="twitter:description" content={metadata.description} />
  <meta name="twitter:image" content={metadata.ogImage} />
</svelte:head>

<!-- Persistent background layer - only for public routes -->
<!-- These elements have view-transition-name to exclude them from page transitions -->
{#if showGlobalBackground}
  <!-- Randomly chosen background component (Clouds or Dots) -->
  {#if backgroundComponent === "clouds"}
    <Clouds style="view-transition-name: background" />
  {:else if backgroundComponent === "dots"}
    <Dots style="view-transition-name: background" />
  {/if}

  <!-- Theme toggle - persistent across page transitions -->
  <div
    class="fixed top-5 right-6 z-50"
    style="view-transition-name: theme-toggle"
  >
    <ThemeToggle />
  </div>
{/if}

<!-- Page content wrapper - this is what transitions between pages -->
<div class="pb-12" style="view-transition-name: page-content">
  {@render children()}
</div>
