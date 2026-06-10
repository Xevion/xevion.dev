<script lang="ts">
  import { css, cx } from "styled-system/css";
  import { flex } from "styled-system/patterns";
  import { telemetry } from "$lib/telemetry";
  import TagList from "./TagList.svelte";
  import type { ApiAdminProject } from "$lib/bindings";

  interface Props {
    project: ApiAdminProject;
    class?: string;
  }

  let { project, class: className }: Props = $props();

  // Internal detail page, when the project has authored content.
  const detailUrl = $derived(
    project.hasDetail ? `/projects/${project.slug}` : null,
  );

  // External target (prefer demo, fall back to GitHub).
  const externalUrl = $derived(
    project.demoUrl ||
      (project.githubRepo ? `https://github.com/${project.githubRepo}` : null),
  );

  // The card's primary click target: the detail page if it exists, else the
  // external link (preserving the original whole-card-is-external behavior).
  const primaryHref = $derived(detailUrl ?? externalUrl);
  const primaryIsExternal = $derived(!detailUrl && !!externalUrl);
  const isLink = $derived(!!primaryHref);

  // When the card goes to a detail page, surface external links as corner pills
  // (they can't nest inside the stretched primary <a>).
  const cornerLinks = $derived(detailUrl ? project.links : []);

  // Primary media (first by display order), used as a faded background.
  const primaryMedia = $derived(project.media?.[0]);
  const hasMedia = $derived(!!primaryMedia);
  const isVideo = $derived(primaryMedia?.mediaType === "video");

  const videoUrl = $derived(
    isVideo ? primaryMedia?.variants.video?.url : undefined,
  );
  const imageUrl = $derived(
    !isVideo && primaryMedia
      ? (primaryMedia.variants.medium?.url ??
          primaryMedia.variants.thumb?.url ??
          primaryMedia.variants.full?.url)
      : undefined,
  );
  const videoPosterUrl = $derived(
    isVideo ? primaryMedia?.variants.poster?.url : undefined,
  );

  let videoElement: HTMLVideoElement | null = $state(null);

  function handleMouseEnter() {
    videoElement?.play();
  }

  function handleMouseLeave() {
    videoElement?.pause();
  }

  function trackPrimaryClick() {
    if (primaryIsExternal && primaryHref) {
      telemetry.track({
        name: "project_interaction",
        properties: {
          action: project.demoUrl ? "demo_click" : "github_click",
          projectSlug: project.slug,
          projectName: project.name,
          targetUrl: primaryHref,
        },
      });
    } else if (detailUrl) {
      telemetry.track({
        name: "project_interaction",
        properties: {
          action: "detail_view",
          projectSlug: project.slug,
          projectName: project.name,
        },
      });
    }
  }

  function trackCornerClick(url: string) {
    telemetry.trackExternalLink(url, "project");
  }

  const mediaBaseStyles = cx(
    "media-mask-fade-left",
    css({
      position: "absolute",
      right: "0",
      top: "0",
      h: "full",
      w: "3/4",
      objectFit: "cover",
      objectPosition: "center",
    }),
  );

  function formatDate(dateString: string): string {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / (1000 * 60));
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffMins < 1) return "just now";
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffHours <= 48) return "yesterday";
    if (diffDays < 30) return `${diffDays}d ago`;
    if (diffDays < 365) return `${Math.floor(diffDays / 30)}mo ago`;
    return `${Math.floor(diffDays / 365)}y ago`;
  }
</script>

<div
  onmouseenter={handleMouseEnter}
  onmouseleave={handleMouseLeave}
  role="article"
  class={cx(
    "group",
    flex({
      direction: "column",
      gap: "2.5",
      position: "relative",
      h: "44",
      rounded: "lg",
      borderWidth: "1px",
      borderColor: "zinc.200",
      bg: "zinc.50",
      p: "3",
      overflow: "hidden",
      _dark: {
        borderColor: "zinc.800",
        bg: "zinc.900/50",
      },
    }),
    isLink &&
      css({
        transition: "all",
        _hover: {
          borderColor: "zinc.300",
          bg: "zinc.100/80",
          _dark: {
            borderColor: "zinc.700",
            bg: "zinc.800/50",
          },
        },
      }),
    className,
  )}
>
  <!-- Background media layer -->
  {#if hasMedia}
    <div
      class={css({
        pointerEvents: "none",
        position: "absolute",
        inset: "0",
        opacity: "0.25",
        transition: "opacity 300ms ease-in-out",
        _groupHover: { opacity: "0.4" },
      })}
      aria-hidden="true"
    >
      {#if isVideo && videoUrl}
        <video
          bind:this={videoElement}
          src={videoUrl}
          poster={videoPosterUrl}
          class={cx(
            mediaBaseStyles,
            css({
              filter: "grayscale(1)",
              transition: "filter 300ms ease-in-out",
              _groupHover: { filter: "grayscale(0)" },
            }),
          )}
          muted
          loop
          playsinline
          preload="metadata"
        ></video>
      {:else if imageUrl}
        <img src={imageUrl} alt="" class={mediaBaseStyles} loading="lazy" />
      {/if}
    </div>
  {/if}

  <!-- Stretched primary link (covers the card; transparent overlay) -->
  {#if isLink && primaryHref}
    <a
      href={primaryHref}
      target={primaryIsExternal ? "_blank" : undefined}
      rel={primaryIsExternal ? "noopener noreferrer" : undefined}
      onclick={trackPrimaryClick}
      aria-label={project.name}
      class={css({ position: "absolute", inset: "0", zIndex: "20" })}
    ></a>
  {/if}

  <!-- External link pills (above the stretched link) -->
  {#if cornerLinks.length > 0}
    <div
      class={cx(
        flex({ gap: "1" }),
        css({ position: "absolute", top: "2", right: "2", zIndex: "30" }),
      )}
    >
      {#each cornerLinks as link (link.url)}
        <a
          href={link.url}
          target="_blank"
          rel="noopener noreferrer"
          onclick={() => trackCornerClick(link.url)}
          class={css({
            px: "1.5",
            py: "0.5",
            rounded: "sm",
            fontSize: "xs",
            fontWeight: "medium",
            bg: "zinc.200/80",
            color: "zinc.700",
            backdropFilter: "blur(2px)",
            transition: "colors",
            _hover: { bg: "zinc.300", color: "zinc.900" },
            _dark: {
              bg: "zinc.800/80",
              color: "zinc.300",
              _hover: { bg: "zinc.700", color: "white" },
            },
          })}
        >
          {link.title ?? "Link"}
        </a>
      {/each}
    </div>
  {/if}

  <!-- Content layer -->
  <div
    class={flex({
      direction: "column",
      gap: "1",
      position: "relative",
      zIndex: "10",
      transition: "opacity 300ms ease-in-out",
      _groupHover: { opacity: "0.8" },
    })}
  >
    <div
      class={flex({ align: "flex-start", justify: "space-between", gap: "2" })}
    >
      <h3
        class={cx(
          css({
            truncate: true,
            fontWeight: "medium",
            fontSize: "lg",
            color: "zinc.900",
            sm: { fontSize: "base" },
            _dark: { color: "zinc.100" },
          }),
          isLink &&
            css({
              transition: "colors",
              _groupHover: {
                color: "zinc.950",
                _dark: { color: "white" },
              },
            }),
        )}
      >
        {project.name}
      </h3>
      <span
        class={css({
          flexShrink: "0",
          color: "zinc.600",
          sm: { fontSize: "0.83rem" },
          _dark: { color: "zinc.300" },
        })}
      >
        {formatDate(project.lastActivity)}
      </span>
    </div>
    <p
      class={css({
        lineClamp: "3",
        lineHeight: "relaxed",
        color: "zinc.600",
        sm: { fontSize: "sm" },
        _dark: { color: "zinc.400" },
      })}
    >
      {project.shortDescription}
    </p>
  </div>

  <!-- Tags layer -->
  <TagList
    tags={project.tags}
    maxRows={2}
    class={css({
      position: "relative",
      zIndex: "10",
      mt: "auto",
      transition: "opacity 300ms ease-in-out",
      _groupHover: { opacity: "0.9" },
    })}
  />
</div>
