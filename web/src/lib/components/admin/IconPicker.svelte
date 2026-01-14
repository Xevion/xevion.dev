<script lang="ts">
  import { SvelteMap } from "svelte/reactivity";
  import { cn } from "$lib/utils";
  import type { IconCollection } from "$lib/types/icons";

  interface Props {
    selectedIcon: string;
    label?: string;
    placeholder?: string;
    class?: string;
  }

  let {
    selectedIcon = $bindable(""),
    label,
    placeholder = "Search icons... (e.g., lucide:home or just home)",
    class: className,
  }: Props = $props();

  let searchQuery = $state("");
  let searchResults = $state<
    Array<{ identifier: string; collection: string; name: string }>
  >([]);
  let collections = $state<IconCollection[]>([]);
  let selectedCollection = $state<string>("all");
  let isLoading = $state(false);
  let showDropdown = $state(false);
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  // Load SVG cache for preview
  let iconSvgCache = new SvelteMap<string, string>();
  let selectedIconSvg = $state<string | null>(null);

  // IntersectionObserver for lazy loading icons
  let observer: IntersectionObserver | null = null;

  // Generate unique ID for accessibility
  const inputId = `iconpicker-${Math.random().toString(36).substring(2, 11)}`;

  // Load collections on mount and setup observer
  $effect(() => {
    loadCollections();
    setupIntersectionObserver();

    return () => {
      if (observer) {
        observer.disconnect();
      }
    };
  });

  // Load selected icon SVG
  $effect(() => {
    if (selectedIcon) {
      loadIconSvg(selectedIcon);
    }
  });

  function setupIntersectionObserver() {
    observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            const identifier = entry.target.getAttribute("data-icon-id");
            if (identifier && !iconSvgCache.has(identifier)) {
              loadIconSvg(identifier);
            }
          }
        }
      },
      {
        root: null,
        rootMargin: "50px",
        threshold: 0.01,
      },
    );
  }

  // Debounced search
  $effect(() => {
    if (searchQuery) {
      if (debounceTimer) {
        clearTimeout(debounceTimer);
      }

      debounceTimer = setTimeout(() => {
        performSearch();
      }, 300);
    } else {
      searchResults = [];
      showDropdown = false;
    }
  });

  async function loadCollections() {
    try {
      const response = await fetch("/api/icons/collections");
      if (response.ok) {
        const data = await response.json();
        collections = data.collections;
      }
    } catch (error) {
      console.error("Failed to load collections:", error);
    }
  }

  async function performSearch() {
    isLoading = true;
    showDropdown = true;

    try {
      // Build query with collection filter if not "all"
      let query = searchQuery;
      if (selectedCollection !== "all" && !query.includes(":")) {
        query = `${selectedCollection}:${query}`;
      }

      const response = await fetch(
        `/api/icons/search?q=${encodeURIComponent(query)}&limit=100`,
      );
      if (response.ok) {
        const data = await response.json();
        searchResults = data.icons;

        // Wait for DOM to update, then observe icon elements
        setTimeout(() => observeIconElements(), 100);
      }
    } catch (error) {
      console.error("Failed to search icons:", error);
    } finally {
      isLoading = false;
    }
  }

  function observeIconElements() {
    if (!observer) return;

    // Find all icon button elements and observe them
    const iconButtons = document.querySelectorAll(`[data-icon-id]`);
    for (const button of iconButtons) {
      observer.observe(button);
    }
  }

  async function loadIconSvg(identifier: string) {
    // Check cache first
    if (iconSvgCache.has(identifier)) {
      if (identifier === selectedIcon) {
        selectedIconSvg = iconSvgCache.get(identifier)!;
      }
      return;
    }

    try {
      const [collection, name] = identifier.split(":");
      const response = await fetch(`/api/icons/${collection}/${name}`);
      if (response.ok) {
        const data = await response.json();
        iconSvgCache.set(identifier, data.svg);
        if (identifier === selectedIcon) {
          selectedIconSvg = data.svg;
        }
      }
    } catch (error) {
      console.error("Failed to load icon SVG:", error);
      if (identifier === selectedIcon) {
        selectedIconSvg = null;
      }
    }
  }

  function selectIcon(identifier: string) {
    selectedIcon = identifier;
    searchQuery = "";
    showDropdown = false;
    loadIconSvg(identifier);
  }

  function handleInputFocus() {
    if (searchQuery && searchResults.length > 0) {
      showDropdown = true;
    }
  }

  function handleInputBlur() {
    setTimeout(() => {
      showDropdown = false;
    }, 200);
  }

  function clearSelection() {
    selectedIcon = "";
    selectedIconSvg = null;
  }
</script>

<div class={cn("space-y-2", className)}>
  {#if label}
    <label for={inputId} class="block text-sm font-medium text-admin-text">
      {label}
    </label>
  {/if}

  <!-- Selected icon preview -->
  {#if selectedIcon}
    <div
      class="flex items-center gap-3 rounded-md border border-admin-border bg-admin-bg-secondary p-3"
    >
      <div
        class="flex size-10 items-center justify-center rounded bg-admin-bg"
        data-icon-container
      >
        {#if selectedIconSvg}
          <!-- eslint-disable-next-line svelte/no-at-html-tags -->
          {@html selectedIconSvg}
        {:else}
          <div
            class="size-6 animate-pulse rounded bg-admin-surface-hover"
          ></div>
        {/if}
      </div>
      <div class="flex-1">
        <p class="text-sm font-medium text-admin-text">{selectedIcon}</p>
      </div>
      <button
        type="button"
        onclick={clearSelection}
        class="rounded px-2 py-1 text-sm text-admin-text-muted hover:bg-admin-surface-hover hover:text-admin-text"
      >
        Clear
      </button>
    </div>
  {/if}

  <!-- Collection tabs -->
  <div class="flex flex-wrap gap-1">
    <button
      type="button"
      class={cn(
        "rounded-md px-3 py-1.5 text-sm font-medium transition-colors",
        selectedCollection === "all"
          ? "bg-admin-accent text-white"
          : "bg-admin-surface text-admin-text-muted hover:bg-admin-surface-hover hover:text-admin-text",
      )}
      onclick={() => (selectedCollection = "all")}
    >
      All
    </button>
    {#each collections as collection (collection.id)}
      <button
        type="button"
        class={cn(
          "rounded-md px-3 py-1.5 text-sm font-medium transition-colors",
          selectedCollection === collection.id
            ? "bg-admin-accent text-white"
            : "bg-admin-surface text-admin-text-muted hover:bg-admin-surface-hover hover:text-admin-text",
        )}
        onclick={() => (selectedCollection = collection.id)}
      >
        {collection.name}
        <span class="ml-1 text-xs opacity-60">({collection.total})</span>
      </button>
    {/each}
  </div>

  <!-- Search input -->
  <div class="relative">
    <input
      id={inputId}
      type="text"
      bind:value={searchQuery}
      {placeholder}
      class="w-full rounded-md border border-admin-border bg-admin-bg-secondary px-3 py-2 text-sm text-admin-text placeholder:text-admin-text-muted focus:border-admin-accent focus:outline-none focus:ring-1 focus:ring-admin-accent"
      onfocus={handleInputFocus}
      onblur={handleInputBlur}
    />

    <!-- Search results dropdown -->
    {#if showDropdown && searchResults.length > 0}
      <div
        class="absolute z-10 mt-1 max-h-96 w-full overflow-auto rounded-md border border-admin-border bg-admin-surface shadow-lg"
      >
        <!-- Grid layout for icons -->
        <div class="grid grid-cols-8 gap-1 p-2">
          {#each searchResults as result (result.identifier)}
            {@const cachedSvg = iconSvgCache.get(result.identifier)}
            <button
              type="button"
              data-icon-id={result.identifier}
              class="group relative flex size-12 items-center justify-center rounded hover:bg-admin-surface-hover"
              onclick={() => selectIcon(result.identifier)}
              title={result.identifier}
            >
              <!-- Lazy load icon SVG via IntersectionObserver -->
              <div class="size-9 text-admin-text" data-icon-container>
                {#if cachedSvg}
                  <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                  {@html cachedSvg}
                {:else}
                  <div
                    class="size-full animate-pulse rounded bg-admin-surface-hover"
                  ></div>
                {/if}
              </div>

              <!-- Tooltip on hover -->
              <div
                class="pointer-events-none absolute -top-8 left-1/2 z-20 hidden -translate-x-1/2 whitespace-nowrap rounded bg-admin-surface border border-admin-border px-2 py-1 text-xs text-admin-text group-hover:block"
              >
                {result.name}
              </div>
            </button>
          {/each}
        </div>

        {#if isLoading}
          <div
            class="border-t border-admin-border p-3 text-center text-sm text-admin-text-muted"
          >
            Loading...
          </div>
        {/if}
      </div>
    {:else if showDropdown && searchQuery && !isLoading}
      <div
        class="absolute z-10 mt-1 w-full rounded-md border border-admin-border bg-admin-surface p-3 text-center text-sm text-admin-text-muted shadow-lg"
      >
        No icons found for "{searchQuery}"
      </div>
    {/if}
  </div>

  <p class="text-xs text-admin-text-muted">
    Tip: Use "collection:search" to filter (e.g., "lucide:home" or
    "simple-icons:react")
  </p>
</div>

<!-- TODO: Future enhancement - Recent/favorite icons -->
<!-- Store recently used icons in localStorage for quick access -->
<!-- Could add "star" button to favorite frequently used icons -->

<style>
  /* Ensure dynamically-injected SVG icons fill their container */
  [data-icon-container] :global(svg) {
    width: 100%;
    height: 100%;
  }
</style>
