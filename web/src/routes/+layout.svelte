<script lang="ts">
  import "@fontsource-variable/inter/wght.css";
  import "@fontsource/hanken-grotesk/900.css";
  import "@fontsource-variable/schibsted-grotesk/wght.css";
  import "overlayscrollbars/overlayscrollbars.css";
  import "../app.css";
  import { OverlayScrollbars } from "overlayscrollbars";
  import { onMount } from "svelte";
  import { themeStore } from "$lib/stores/theme.svelte";

  let { children, data } = $props();

  const defaultMetadata = {
    title: "Xevion.dev",
    description:
      "The personal website of Xevion, a full-stack software developer.",
    ogImage: "/api/og/home.png",
    url: "https://xevion.dev",
  };

  const metadata = $derived(data?.metadata ?? defaultMetadata);

  onMount(() => {
    // Initialize theme store
    themeStore.init();

    // Initialize overlay scrollbars on the body element
    const osInstance = OverlayScrollbars(document.body, {
      scrollbars: {
        autoHide: "leave",
        autoHideDelay: 800,
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

{@render children()}
