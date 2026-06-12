<script lang="ts">
  import { pushState } from "$app/navigation";
  import { page } from "$app/state";
  import { flip } from "svelte/animate";
  import { cubicOut } from "svelte/easing";
  import ProjectCard from "$lib/components/ProjectCard.svelte";
  import ProjectRow from "$lib/components/ProjectRow.svelte";
  import ProjectFilter from "$lib/components/ProjectFilter.svelte";
  import DiscordProfileModal from "$lib/components/DiscordProfileModal.svelte";
  import Icon from "$lib/components/Icon.svelte";
  import { telemetry } from "$lib/telemetry";
  import { featuredSlugs } from "$lib/project-display";
  import type { PageData } from "./$types";
  import type { ApiAdminProject } from "$lib/bindings";
  import MaterialSymbolsVpnKey from "~icons/material-symbols/vpn-key";
  import { css, cx } from "styled-system/css";
  import { flex, wrap, grid } from "styled-system/patterns";

  let { data }: { data: PageData } = $props();
  const projects = $derived(data.projects);
  const socialLinks = $derived(data.socialLinks);

  const visibleSocialLinks = $derived(
    socialLinks.filter((link) => link.visible),
  );

  function openDiscordModal(username: string) {
    pushState("", { discordModal: { open: true, username } });
  }

  function trackSocialClick(url: string) {
    telemetry.trackExternalLink(url, "social");
  }

  // Smart filter state.
  let selected = $state<string[]>([]);
  const filterActive = $derived(selected.length > 0);

  // Rank against the selected facet set: exact-all first, then by match strength,
  // then original (recency) order. Non-matches stay in the list but dim.
  type Ranked = { p: ApiAdminProject; i: number; matched: number };
  const ranked = $derived.by<Ranked[]>(() => {
    return projects
      .map((p, i) => {
        const names = new Set(p.tags.map((t) => t.name));
        const matched = selected.reduce(
          (n, s) => n + (names.has(s) ? 1 : 0),
          0,
        );
        const exact = selected.length > 0 && matched === selected.length;
        return { p, i, matched, exact };
      })
      .sort((a, b) => {
        if (a.exact !== b.exact) return a.exact ? -1 : 1;
        if (a.matched !== b.matched) return b.matched - a.matched;
        return a.i - b.i;
      });
  });

  const orderedProjects = $derived(ranked.map((r) => r.p));
  const dimSet = $derived(
    new Set(
      filterActive
        ? ranked.filter((r) => r.matched === 0).map((r) => r.p.slug)
        : [],
    ),
  );
  const matchCount = $derived(ranked.filter((r) => r.matched > 0).length);

  // Hybrid: when idle, the top 2 featured render as covers and the rest as rows.
  // When filtering, everything collapses to a single ranked row list.
  const featured = $derived(
    filterActive ? new Set<string>() : featuredSlugs(projects),
  );
  const featuredProjects = $derived(
    orderedProjects.filter((p) => featured.has(p.slug)),
  );
  const restProjects = $derived(
    orderedProjects.filter((p) => !featured.has(p.slug)),
  );

  function toggleFacet(name: string) {
    selected = selected.includes(name)
      ? selected.filter((x) => x !== name)
      : [...selected, name];
  }
  function clearFilter() {
    selected = [];
  }

  const socialBtnClass = flex({
    align: "center",
    columnGap: "1.5",
    px: "1.5",
    py: "1",
    rounded: "sm",
    bg: "zinc.100",
    shadow: "sm",
    transition: "colors",
    cursor: "pointer",
    _dark: { bg: "zinc.900" },
    _hover: { bg: "zinc.200", _dark: { bg: "zinc.800" } },
    _focusVisible: {
      outline: "none",
      ringWidth: "2px",
      ringColor: "zinc.400",
      _dark: { ringColor: "zinc.500" },
    },
  });

  const socialIconClass = css({
    color: "zinc.600",
    _dark: { color: "zinc.300" },
  });

  const socialLabelClass = css({
    whiteSpace: "nowrap",
    fontSize: "sm",
    color: "zinc.800",
    _dark: { color: "zinc.100" },
  });

  const columnClass = css({ maxW: "42rem", w: "full", px: "6" });
</script>

<main
  class={cx(
    "page-main",
    css({ overflowX: "hidden", fontFamily: "schibsted", pb: "12" }),
  )}
>
  <div class={flex({ direction: "column", align: "center", pt: "14" })}>
    <div
      class={cx(
        columnClass,
        css({
          borderBottomWidth: "1px",
          borderColor: "zinc.200",
          divideY: "1px",
          divideColor: "zinc.200",
          _dark: { borderColor: "zinc.700", divideColor: "zinc.700" },
        }),
      )}
    >
      <div class={flex({ direction: "column", pb: "4" })}>
        <span
          class={css({
            fontSize: "2xl",
            fontWeight: "bold",
            color: "zinc.900",
            _dark: { color: "white" },
            sm: { fontSize: "3xl" },
          })}>{data.settings.identity.displayName},</span
        >
        <span
          class={css({
            fontSize: "xl",
            fontWeight: "normal",
            color: "zinc.600",
            _dark: { color: "zinc.400" },
            sm: { fontSize: "2xl" },
          })}
        >
          {data.settings.identity.occupation}
        </span>
      </div>

      <div
        class={css({
          py: "4",
          color: "zinc.700",
          _dark: { color: "zinc.200" },
        })}
      >
        <p class={css({ sm: { fontSize: "0.95em" }, whiteSpace: "pre-line" })}>
          {data.settings.identity.bio}
        </p>
      </div>

      <div class={css({ py: "3" })}>
        <span class={css({ color: "zinc.700", _dark: { color: "zinc.200" } })}
          >Connect with me</span
        >
        <div class={wrap({ gap: "2", pl: "3", pt: "3", pb: "2" })}>
          {#each visibleSocialLinks as link (link.id)}
            {#if link.platform === "github" || link.platform === "linkedin"}
              <a
                href={link.value}
                onclick={() => trackSocialClick(link.value)}
                class={socialBtnClass}
              >
                <Icon icon={link.icon} size="4" class={socialIconClass} />
                <span class={socialLabelClass}>{link.label}</span>
              </a>
            {:else if link.platform === "discord"}
              <button
                type="button"
                class={socialBtnClass}
                onclick={() => {
                  trackSocialClick(`discord:${link.value}`);
                  openDiscordModal(link.value);
                }}
              >
                <Icon icon={link.icon} size="4" class={socialIconClass} />
                <span class={socialLabelClass}>{link.label}</span>
              </button>
            {:else if link.platform === "email"}
              <a
                href="mailto:{link.value}"
                onclick={() => trackSocialClick(`mailto:${link.value}`)}
                class={socialBtnClass}
              >
                <Icon icon={link.icon} size="4.5" class={socialIconClass} />
                <span class={socialLabelClass}>{link.label}</span>
              </a>
            {/if}
          {/each}
          <a href="/pgp" class={socialBtnClass}>
            <MaterialSymbolsVpnKey
              class={css({
                w: "4.5",
                h: "4.5",
                color: "zinc.600",
                _dark: { color: "zinc.300" },
              })}
            />
            <span class={socialLabelClass}>PGP Key</span>
          </a>
        </div>
      </div>
    </div>

    <div class={cx(columnClass, css({ pt: "18px" }))}>
      <ProjectFilter
        {projects}
        {selected}
        {matchCount}
        onToggle={toggleFacet}
        onClear={clearFilter}
      />

      {#if filterActive}
        <div>
          {#each orderedProjects as project (project.slug)}
            <div animate:flip={{ duration: 460, easing: cubicOut }}>
              <ProjectRow {project} dim={dimSet.has(project.slug)} />
            </div>
          {/each}
        </div>
      {:else}
        {#if featuredProjects.length > 0}
          <div class={grid({ columns: 2, gap: "12px" })}>
            {#each featuredProjects as project (project.slug)}
              <ProjectCard {project} />
            {/each}
          </div>
        {/if}
        <div class={css({ mt: "14px" })}>
          {#each restProjects as project (project.slug)}
            <div animate:flip={{ duration: 460, easing: cubicOut }}>
              <ProjectRow {project} />
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</main>

{#if page.state.discordModal?.open}
  <DiscordProfileModal
    username={page.state.discordModal.username}
    onclose={() => history.back()}
  />
{/if}
