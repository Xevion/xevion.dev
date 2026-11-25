<script lang="ts">
  import { SvelteURLSearchParams } from "svelte/reactivity";
  import { css } from "styled-system/css";
  import { hstack } from "styled-system/patterns";
  import { page } from "$app/state";
  import {
    adminCardClass,
    adminInputBase,
    labelClass,
    pageDescriptionClass,
    pageTitleClass,
  } from "#lib/styles/admin.ts";
  import type { PageData } from "./$types";

  let { data }: { data: PageData } = $props();

  type CardType = "index" | "projects" | "project";

  let cardType = $state<CardType>(
    (page.url.searchParams.get("type") as CardType | null) ?? "index",
  );
  let selectedProject = $state<string | undefined>(
    page.url.searchParams.get("id") ?? undefined,
  );
  const projectId = $derived(selectedProject ?? data.projects[0]?.id ?? "");
  let nonce = $state(0);

  const imageUrl = $derived.by(() => {
    const params = new SvelteURLSearchParams({ type: cardType });
    if (cardType === "project" && projectId) params.set("id", projectId);
    params.set("_", String(nonce));
    return `/admin/og/image?${params}`;
  });
  const ready = $derived(cardType !== "project" || projectId !== "");
</script>

<svelte:head>
  <title>OG Preview | Admin</title>
</svelte:head>

<div class={css({ spaceY: "6" })}>
  <div>
    <h1 class={pageTitleClass}>OG Preview</h1>
    <p class={pageDescriptionClass}>
      Renders the card on demand without writing to storage.
    </p>
  </div>

  <div class={adminCardClass}>
    <div class={hstack({ gap: "4", alignItems: "flex-end", flexWrap: "wrap" })}>
      <div>
        <label for="og-type" class={labelClass}>Card</label>
        <select id="og-type" class={adminInputBase} bind:value={cardType}>
          <option value="index">Index</option>
          <option value="projects">Projects</option>
          <option value="project">Project</option>
        </select>
      </div>
      {#if cardType === "project"}
        <div>
          <label for="og-project" class={labelClass}>Project</label>
          <select
            id="og-project"
            class={adminInputBase}
            bind:value={selectedProject}
          >
            {#each data.projects as project (project.id)}
              <option value={project.id}>
                {project.name}{project.hidden ? " (hidden)" : ""}
              </option>
            {/each}
          </select>
        </div>
      {/if}
      <button
        type="button"
        class={css({
          px: "4",
          py: "2",
          rounded: "md",
          bg: "admin.accent",
          color: "white",
          cursor: "pointer",
        })}
        onclick={() => nonce++}
      >
        Refresh
      </button>
    </div>
  </div>

  {#if ready}
    <img
      src={imageUrl}
      alt="OG card preview"
      width="1200"
      height="630"
      class={css({
        maxW: "full",
        h: "auto",
        rounded: "md",
        borderWidth: "1px",
        borderColor: "admin.border",
      })}
    />
  {/if}
</div>
