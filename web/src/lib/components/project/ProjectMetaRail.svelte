<script lang="ts">
  import { css } from "styled-system/css";
  import IconGithub from "~icons/simple-icons/github";
  import IconExternal from "~icons/lucide/arrow-up-right";
  import IconLock from "~icons/lucide/lock";
  import type { ApiProjectDetail } from "$lib/bindings";
  import { formatCreated, formatAge, statusMeta } from "$lib/project-display";

  interface Props {
    project: ApiProjectDetail;
    /** Server-seeded clock so relative ages match across SSR/hydration. */
    now?: number;
    onLink?: (url: string) => void;
  }

  let { project, now, onLink }: Props = $props();

  const status = $derived(statusMeta(project.status));
  const stack = $derived(project.tags);

  const repoUrl = $derived(
    project.githubRepo ? `https://github.com/${project.githubRepo}` : null,
  );
  // Action layout is decided by data: demo > open repo > closed lock.
  const showDemo = $derived(!!project.demoUrl);
  const showRepoRow = $derived(
    !project.demoUrl && !!repoUrl && !project.sourceClosed,
  );
  const showClosed = $derived(
    !project.demoUrl && !showRepoRow && project.sourceClosed,
  );

  const factLabel = css({ textStyle: "label.micro" });
  const fact = css({
    display: "flex",
    flexDirection: "column",
    gap: "4px",
  });
</script>

<aside class="rd-rail">
  <div
    class={css({
      display: "grid",
      gridTemplateColumns: "1fr 1fr",
      gap: "15px 14px",
    })}
  >
    <div class={fact}>
      <span class={factLabel}>Type</span>
      <span
        class={css({
          fontSize: "bodySm",
          color: "zinc.800",
          _dark: { color: "zinc.200" },
        })}
      >
        {project.projectType ?? "—"}
      </span>
    </div>
    <div class={fact}>
      <span class={factLabel}>Status</span>
      <span
        class={css({ textStyle: "label.status" })}
        style="color: {status.color}"
      >
        {status.label}
      </span>
    </div>
    <div class={fact}>
      <span class={factLabel}>Created</span>
      <span
        class={css({
          fontSize: "bodySm",
          color: "zinc.800",
          _dark: { color: "zinc.200" },
        })}
      >
        {formatCreated(project.createdAt)}
      </span>
    </div>
    <div class={fact}>
      <span class={factLabel}>Last active</span>
      <span
        class={css({
          fontSize: "bodySm",
          color: "zinc.800",
          _dark: { color: "zinc.200" },
        })}
      >
        {formatAge(project.lastActivity, now)}
      </span>
    </div>
  </div>

  <div
    class={css({
      mt: "16px",
      display: "flex",
      flexDirection: "column",
      gap: "8px",
    })}
  >
    {#if showDemo}
      <a
        href={project.demoUrl}
        target="_blank"
        rel="noopener noreferrer"
        onclick={() => onLink?.(project.demoUrl!)}
        class={css({
          display: "inline-flex",
          alignItems: "center",
          justifyContent: "center",
          gap: "8px",
          p: "11px 14px",
          rounded: "9px",
          bg: "var(--accent)",
          color: "var(--accent-ink)",
          textDecoration: "none",
          fontSize: "14px",
          fontWeight: "600",
          boxShadow:
            "0 8px 18px -10px color-mix(in srgb, var(--accent) 80%, transparent)",
          transition: "transform .14s ease, filter .14s ease",
          _hover: { transform: "translateY(-1px)", filter: "brightness(1.05)" },
        })}
      >
        Open live demo
        <IconExternal class={css({ w: "14px", h: "14px" })} />
      </a>
      {#if repoUrl && !project.sourceClosed}
        <a
          href={repoUrl}
          target="_blank"
          rel="noopener noreferrer"
          onclick={() => onLink?.(repoUrl)}
          class={css({
            display: "inline-flex",
            alignItems: "center",
            justifyContent: "center",
            gap: "8px",
            p: "9px 14px",
            rounded: "9px",
            bg: "surface",
            color: "zinc.700",
            textDecoration: "none",
            fontSize: "bodySm",
            fontWeight: "500",
            borderWidth: "1px",
            borderColor: "zinc.200",
            _dark: { color: "zinc.300", borderColor: "zinc.700" },
          })}
        >
          <IconGithub class={css({ w: "14px", h: "14px" })} />
          Source
        </a>
      {/if}
    {:else if showRepoRow && repoUrl}
      <a
        href={repoUrl}
        target="_blank"
        rel="noopener noreferrer"
        onclick={() => onLink?.(repoUrl)}
        class={css({
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          gap: "8px",
          p: "10px 13px",
          rounded: "9px",
          borderWidth: "1px",
          borderColor: "zinc.200",
          bg: "surface",
          color: "zinc.700",
          textDecoration: "none",
          fontSize: "bodySm",
          fontWeight: "500",
          transition: "border-color .14s ease",
          _hover: { borderColor: "zinc.300" },
          _dark: { color: "zinc.300", borderColor: "zinc.700" },
        })}
      >
        <span
          class={css({
            display: "inline-flex",
            alignItems: "center",
            gap: "8px",
          })}
        >
          <IconGithub class={css({ w: "14px", h: "14px" })} />
          View source
        </span>
        <span
          class={css({ fontFamily: "geist", fontSize: "meta", opacity: "0.5" })}
          >repo</span
        >
      </a>
    {:else if showClosed}
      <div
        class={css({
          display: "flex",
          alignItems: "center",
          gap: "8px",
          p: "10px 13px",
          rounded: "9px",
          borderWidth: "1px",
          borderStyle: "dashed",
          borderColor: "zinc.300",
          bg: "surface.secondary",
          color: "zinc.600",
          fontSize: "13px",
          fontFamily: "geist",
          _dark: { borderColor: "zinc.600", color: "zinc.400" },
        })}
      >
        <IconLock class={css({ w: "14px", h: "14px", color: "zinc.500" })} />
        Source &middot; closed
      </div>
    {/if}
  </div>

  {#if stack.length > 0}
    <div
      class={css({
        borderTopWidth: "1px",
        borderColor: "border.hairline",
        mt: "16px",
        pt: "14px",
      })}
    >
      <span class={factLabel}>Built with</span>
      <div
        class={css({
          display: "flex",
          flexDirection: "column",
          gap: "7px",
          mt: "9px",
        })}
      >
        {#each stack as tag (tag.id)}
          <span
            class={css({
              display: "inline-flex",
              alignItems: "center",
              gap: "8px",
              fontSize: "13px",
              color: "zinc.700",
              _dark: { color: "zinc.300" },
            })}
          >
            <span
              class={css({
                w: "6px",
                h: "6px",
                rounded: "2px",
                flexShrink: "0",
                bg: "var(--accent)",
              })}
            ></span>
            {tag.name}
          </span>
        {/each}
      </div>
    </div>
  {/if}
</aside>
