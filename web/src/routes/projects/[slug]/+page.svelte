<script lang="ts">
  import { telemetry } from "$lib/telemetry";
  import ProjectCover from "$lib/components/ProjectCover.svelte";
  import { morph } from "$lib/stores/morph.svelte";
  import {
    accentOf,
    detectLanguage,
    statusMeta,
    formatAge,
    formatCreated,
    isStackTag,
    tagColor,
  } from "$lib/project-display";
  import type { PageData } from "./$types";
  import type { ApiAdminProject } from "$lib/bindings";
  import { css, cx } from "styled-system/css";

  let { data }: { data: PageData } = $props();
  const project = $derived(data.project);
  const allProjects = $derived(data.projects);

  const accent = $derived(accentOf(project));
  const language = $derived(detectLanguage(project));
  const status = $derived(statusMeta(project.status));
  const stack = $derived(project.tags.filter(isStackTag));

  // Sidebar links: live/demo (primary, solid) + repo (outline) + any extras.
  type SideLink = {
    title: string;
    url: string;
    kind: "demo" | "github" | "other";
  };
  const sideLinks = $derived.by<SideLink[]>(() => {
    const links: SideLink[] = [];
    if (project.demoUrl)
      links.push({ title: "Live", url: project.demoUrl, kind: "demo" });
    if (project.githubRepo)
      links.push({
        title: "GitHub",
        url: `https://github.com/${project.githubRepo}`,
        kind: "github",
      });
    for (const l of project.links) {
      if (!links.some((s) => s.url === l.url))
        links.push({ title: l.title ?? "Link", url: l.url, kind: "other" });
    }
    return links;
  });

  // Related: most shared tags. Pager: sequential by list order.
  const related = $derived.by(() => {
    const tagNames = new Set(project.tags.map((t) => t.name));
    return allProjects
      .filter((p) => p.slug !== project.slug)
      .map((p) => ({
        p,
        score: p.tags.filter((t) => tagNames.has(t.name)).length,
      }))
      .sort((a, b) => b.score - a.score)
      .slice(0, 3)
      .map((x) => x.p);
  });
  const pagerIndex = $derived(
    allProjects.findIndex((p) => p.slug === project.slug),
  );
  const prevProject = $derived(
    pagerIndex > 0 ? allProjects[pagerIndex - 1] : null,
  );
  const nextProject = $derived(
    pagerIndex >= 0 && pagerIndex < allProjects.length - 1
      ? allProjects[pagerIndex + 1]
      : null,
  );

  // Mark this project as the morph target so back-navigation reverses into its card.
  $effect(() => {
    morph.slug = project.slug;
  });

  function trackLink(url: string) {
    telemetry.trackExternalLink(url, "project");
  }
  function openRelated(p: ApiAdminProject) {
    morph.slug = p.slug;
    telemetry.track({
      name: "project_interaction",
      properties: {
        action: "detail_view",
        projectSlug: p.slug,
        projectName: p.name,
      },
    });
  }

  const proseClass = css({
    color: "zinc.700",
    _dark: { color: "zinc.300" },
    "& h2": {
      fontSize: "19px",
      fontWeight: "700",
      letterSpacing: "-0.01em",
      color: "zinc.900",
      m: "30px 0 10px",
      _dark: { color: "white" },
    },
    "& h3": {
      fontSize: "17px",
      fontWeight: "700",
      color: "zinc.900",
      m: "24px 0 8px",
      _dark: { color: "zinc.100" },
    },
    "& h4": {
      fontSize: "15px",
      fontWeight: "600",
      color: "zinc.900",
      m: "20px 0 6px",
      _dark: { color: "zinc.100" },
    },
    "& p": {
      fontSize: "15.5px",
      lineHeight: "1.72",
      color: "zinc.700",
      m: "0 0 14px",
      textWrap: "pretty",
      _dark: { color: "zinc.300" },
    },
    "& ul": {
      listStyle: "none",
      p: "0",
      m: "0 0 16px",
      display: "flex",
      flexDirection: "column",
      gap: "8px",
    },
    "& ul > li": {
      position: "relative",
      pl: "20px",
      fontSize: "15px",
      lineHeight: "1.6",
      color: "zinc.700",
      _dark: { color: "zinc.300" },
    },
    "& ul > li::before": {
      content: '""',
      position: "absolute",
      left: "2px",
      top: "9px",
      w: "5px",
      h: "5px",
      rounded: "1px",
      bg: "#c2410c",
    },
    "& ol": {
      listStyle: "decimal",
      pl: "22px",
      m: "0 0 16px",
      display: "flex",
      flexDirection: "column",
      gap: "8px",
    },
    "& ol > li": {
      fontSize: "15px",
      lineHeight: "1.6",
      color: "zinc.700",
      _dark: { color: "zinc.300" },
    },
    "& a": {
      color: "blue.600",
      textDecoration: "underline",
      _dark: { color: "blue.400" },
    },
    // Inline code (the in-block Shiki code is reset in the component style block).
    "& code": {
      fontFamily: "geist",
      fontSize: "0.86em",
      bg: "zinc.100",
      borderWidth: "1px",
      borderColor: "zinc.200",
      rounded: "4px",
      px: "5px",
      py: "1px",
      color: "#b7410e",
      whiteSpace: "nowrap",
      _dark: { bg: "zinc.800", borderColor: "zinc.700", color: "#e8804f" },
    },
    "& kbd": {
      fontFamily: "geist",
      fontSize: "11.5px",
      display: "inline-flex",
      alignItems: "center",
      minH: "18px",
      px: "6px",
      py: "1px",
      borderWidth: "1px",
      borderBottomWidth: "2px",
      borderColor: "zinc.300",
      rounded: "5px",
      bg: "surface",
      color: "zinc.700",
      shadow: "0 1px 0 rgba(24,24,27,.04)",
      whiteSpace: "nowrap",
      _dark: { borderColor: "zinc.600", color: "zinc.300" },
    },
    "& blockquote": {
      position: "relative",
      m: "24px 0",
      p: "4px 0 4px 26px",
      borderLeftWidth: "3px",
      borderColor: "#c2410c",
      fontSize: "18px",
      lineHeight: "1.55",
      color: "zinc.800",
      textWrap: "pretty",
      _dark: { color: "zinc.200" },
    },
    "& blockquote::before": {
      content: '"\\201C"',
      position: "absolute",
      left: "12px",
      top: "6px",
      fontSize: "26px",
      lineHeight: "1",
      color: "zinc.300",
      fontFamily: "Georgia, serif",
      _dark: { color: "zinc.600" },
    },
    "& hr": {
      borderColor: "zinc.200",
      my: "6",
      _dark: { borderColor: "zinc.700" },
    },
    "& img": { maxW: "full", rounded: "md", my: "4" },
  });

  const factLabel = css({
    fontFamily: "geist",
    fontSize: "10px",
    letterSpacing: "0.07em",
    textTransform: "uppercase",
    color: "zinc.400",
  });
  const factValue = css({
    fontSize: "13.5px",
    color: "zinc.800",
    display: "inline-flex",
    alignItems: "center",
    gap: "7px",
    _dark: { color: "zinc.200" },
  });

  const pagerLabel = css({
    fontFamily: "geist",
    fontSize: "11px",
    color: "zinc.400",
    display: "inline-flex",
    alignItems: "center",
    gap: "6px",
  });
  const pagerName = css({
    fontSize: "15px",
    fontWeight: "600",
    color: "zinc.900",
    _dark: { color: "zinc.50" },
  });
  const pagerClass = (isNext: boolean) =>
    cx(
      css({
        display: "flex",
        flexDirection: "column",
        gap: "3px",
        p: "8px 10px",
        rounded: "8px",
        maxW: "48%",
        textDecoration: "none",
        transition: "background .14s ease",
        _hover: { bg: "surface.secondary/80" },
      }),
      isNext
        ? css({ alignItems: "flex-end", textAlign: "right" })
        : css({ alignItems: "flex-start" }),
    );
</script>

<svelte:head>
  <title>{project.name} | Xevion</title>
  <meta name="description" content={project.shortDescription} />
</svelte:head>

<main
  class={cx(
    "page-main",
    css({ overflowX: "hidden", fontFamily: "schibsted", pb: "20" }),
  )}
>
  <div class={css({ display: "flex", justifyContent: "center", pt: "14" })}>
    <div class={css({ maxW: "940px", w: "full", px: "24px" })}>
      <a
        href="/"
        class={cx(
          "group",
          css({
            display: "inline-flex",
            alignItems: "center",
            gap: "8px",
            mb: "22px",
            p: "7px 14px 7px 11px",
            rounded: "full",
            borderWidth: "1px",
            borderColor: "zinc.200",
            bg: "surface/80",
            color: "zinc.700",
            fontSize: "13px",
            textDecoration: "none",
            transition:
              "border-color .15s ease, background .15s ease, color .15s ease",
            _hover: {
              borderColor: "zinc.300",
              color: "zinc.900",
              _dark: { color: "zinc.50" },
            },
            _dark: { borderColor: "zinc.700", color: "zinc.300" },
          }),
        )}
      >
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.2"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
          class={css({
            transition: "transform .15s ease",
            _groupHover: { transform: "translateX(-2px)" },
          })}
        >
          <path d="M11 5 4 12l7 7M4 12h15" />
        </svg>
        Back to index
      </a>

      <div
        class={css({
          position: "relative",
          h: "210px",
          rounded: "6px",
          overflow: "hidden",
          borderWidth: "1px",
          borderColor: "zinc.200",
          bg: "surface.secondary",
          _dark: { borderColor: "zinc.700" },
        })}
        style="view-transition-name: project-cover"
      >
        <ProjectCover
          seed={project.name}
          {accent}
          cols={32}
          rows={9}
          cell={22}
          monogram={project.name[0]}
        />
      </div>

      <header class={css({ mt: "24px" })}>
        <div
          class={css({
            display: "flex",
            alignItems: "center",
            gap: "12px",
            flexWrap: "wrap",
          })}
        >
          <h1
            class={css({
              fontSize: "36px",
              fontWeight: "700",
              letterSpacing: "-0.02em",
              color: "zinc.900",
              _dark: { color: "white" },
            })}
            style="view-transition-name: project-title"
          >
            {project.name}
          </h1>
          <span
            class={css({
              fontFamily: "geist",
              fontSize: "11px",
              fontWeight: "500",
              letterSpacing: "0.09em",
              textTransform: "uppercase",
            })}
            style="color: {status.color}"
          >
            {status.label}
          </span>
        </div>
        <p
          class={css({
            mt: "11px",
            fontSize: "18.5px",
            lineHeight: "1.5",
            color: "zinc.600",
            maxW: "620px",
            textWrap: "pretty",
            _dark: { color: "zinc.400" },
          })}
        >
          {project.shortDescription}
        </p>
      </header>

      <div class="rd-detail-grid">
        <div class={css({ minW: "0" })}>
          {#if data.html}
            <div class={cx("project-detail", proseClass)}>
              <!-- eslint-disable-next-line svelte/no-at-html-tags -- server-rendered, sanitized TipTap output -->
              {@html data.html}
            </div>
          {/if}

          {#if project.media.length > 0}
            <div class={css({ mt: "26px" })}>
              <h2
                class={css({
                  fontFamily: "geist",
                  fontSize: "13px",
                  letterSpacing: "0.06em",
                  textTransform: "uppercase",
                  color: "zinc.400",
                  m: "0 0 14px",
                })}
              >
                Gallery
              </h2>
              <div
                class={css({
                  display: "grid",
                  gridTemplateColumns:
                    project.media.length > 1 ? "1fr 1fr" : "1fr",
                  gap: "12px",
                })}
              >
                {#each project.media as m (m.id)}
                  {@const img =
                    m.variants.medium?.url ??
                    m.variants.full?.url ??
                    m.variants.thumb?.url}
                  <div
                    class={css({
                      position: "relative",
                      aspectRatio: "16 / 10",
                      rounded: "8px",
                      overflow: "hidden",
                      borderWidth: "1px",
                      borderColor: "zinc.100",
                      bg: "surface.secondary",
                      _dark: { borderColor: "zinc.800" },
                    })}
                  >
                    {#if img}
                      <img
                        src={img}
                        alt=""
                        loading="lazy"
                        class={css({
                          position: "absolute",
                          inset: "0",
                          w: "full",
                          h: "full",
                          objectFit: "cover",
                        })}
                      />
                    {/if}
                  </div>
                {/each}
              </div>
            </div>
          {/if}
        </div>

        <aside class="rd-detail-side">
          <div
            class={css({
              display: "flex",
              flexDirection: "column",
              gap: "16px",
            })}
          >
            <div
              class={css({
                display: "grid",
                gridTemplateColumns: "1fr 1fr",
                gap: "15px 14px",
              })}
            >
              <div
                class={css({
                  display: "flex",
                  flexDirection: "column",
                  gap: "4px",
                })}
              >
                <span class={factLabel}>Language</span>
                <span class={factValue}>
                  {#if language}
                    <span
                      class={css({
                        w: "8px",
                        h: "8px",
                        rounded: "full",
                        flexShrink: "0",
                      })}
                      style="background: {language.color}"
                    ></span>
                    {language.name}
                  {:else}
                    —
                  {/if}
                </span>
              </div>
              <div
                class={css({
                  display: "flex",
                  flexDirection: "column",
                  gap: "4px",
                })}
              >
                <span class={factLabel}>Status</span>
                <span class={factValue}>{status.label}</span>
              </div>
              <div
                class={css({
                  display: "flex",
                  flexDirection: "column",
                  gap: "4px",
                })}
              >
                <span class={factLabel}>Created</span>
                <span class={factValue}>{formatCreated(project.createdAt)}</span
                >
              </div>
              <div
                class={css({
                  display: "flex",
                  flexDirection: "column",
                  gap: "4px",
                })}
              >
                <span class={factLabel}>Activity</span>
                <span class={factValue}>{formatAge(project.lastActivity)}</span>
              </div>
            </div>

            {#if sideLinks.length > 0}
              <div
                class={css({
                  display: "flex",
                  flexDirection: "column",
                  gap: "8px",
                })}
              >
                {#each sideLinks as link (link.url)}
                  {@const isPrimary = link.kind === "demo"}
                  <a
                    href={link.url}
                    target="_blank"
                    rel="noopener noreferrer"
                    onclick={() => trackLink(link.url)}
                    class={cx(
                      "rd-sidelink",
                      css({
                        display: "flex",
                        alignItems: "center",
                        justifyContent: "space-between",
                        gap: "8px",
                        p: "8px 12px",
                        rounded: "7px",
                        borderWidth: "1px",
                        fontSize: "13.5px",
                        fontWeight: "500",
                        textDecoration: "none",
                        transition: "box-shadow .14s ease, transform .14s ease",
                        _hover: { transform: "translateY(-1px)" },
                      }),
                      isPrimary
                        ? css({
                            borderColor: "zinc.900",
                            bg: "zinc.900",
                            color: "white",
                            _dark: {
                              borderColor: "zinc.100",
                              bg: "zinc.100",
                              color: "zinc.900",
                            },
                          })
                        : css({
                            borderColor: "zinc.200",
                            bg: "surface",
                            color: "zinc.700",
                            _dark: {
                              borderColor: "zinc.700",
                              color: "zinc.300",
                            },
                          }),
                    )}
                  >
                    <span
                      class={css({
                        display: "inline-flex",
                        alignItems: "center",
                        gap: "8px",
                      })}
                    >
                      {#if link.kind === "github"}
                        <svg
                          width="14"
                          height="14"
                          viewBox="0 0 24 24"
                          fill="currentColor"
                          aria-hidden="true"
                        >
                          <path
                            d="M12 2C6.48 2 2 6.58 2 12.25c0 4.53 2.87 8.37 6.84 9.73.5.1.68-.22.68-.49l-.01-1.7c-2.78.62-3.37-1.37-3.37-1.37-.46-1.18-1.11-1.5-1.11-1.5-.91-.63.07-.62.07-.62 1 .07 1.53 1.06 1.53 1.06.89 1.56 2.34 1.11 2.91.85.09-.66.35-1.11.63-1.36-2.22-.26-4.55-1.14-4.55-5.07 0-1.12.39-2.03 1.03-2.75-.1-.26-.45-1.3.1-2.71 0 0 .84-.28 2.75 1.05a9.3 9.3 0 0 1 5 0c1.91-1.33 2.75-1.05 2.75-1.05.55 1.41.2 2.45.1 2.71.64.72 1.03 1.63 1.03 2.75 0 3.94-2.34 4.81-4.57 5.06.36.32.68.94.68 1.9l-.01 2.82c0 .27.18.6.69.49A10.26 10.26 0 0 0 22 12.25C22 6.58 17.52 2 12 2Z"
                          />
                        </svg>
                      {:else}
                        <svg
                          width="13"
                          height="13"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="2.2"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          aria-hidden="true"
                        >
                          <path d="M7 17 17 7M9 7h8v8" />
                        </svg>
                      {/if}
                      {link.title}
                    </span>
                    <span
                      class={css({
                        opacity: "0.5",
                        fontFamily: "geist",
                        fontSize: "11px",
                      })}
                    >
                      {link.kind === "github"
                        ? "repo"
                        : link.kind === "demo"
                          ? "live"
                          : "link"}
                    </span>
                  </a>
                {/each}
              </div>
            {/if}

            {#if stack.length > 0}
              <div
                class={css({
                  borderTopWidth: "1px",
                  borderColor: "zinc.100",
                  pt: "15px",
                  display: "flex",
                  flexDirection: "column",
                  gap: "9px",
                  _dark: { borderColor: "zinc.800" },
                })}
              >
                <span class={factLabel}>Built with</span>
                <div
                  class={css({
                    display: "flex",
                    flexDirection: "column",
                    gap: "7px",
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
                        })}
                        style="background: {tagColor(tag)}"
                      ></span>
                      {tag.name}
                    </span>
                  {/each}
                </div>
              </div>
            {/if}
          </div>
        </aside>
      </div>

      {#if related.length > 0}
        <div
          class={css({
            mt: "30px",
            pt: "28px",
            borderTopWidth: "1px",
            borderColor: "zinc.100",
            _dark: { borderColor: "zinc.800" },
          })}
        >
          <h2
            class={css({
              fontFamily: "geist",
              fontSize: "13px",
              letterSpacing: "0.06em",
              textTransform: "uppercase",
              color: "zinc.400",
              m: "0 0 16px",
            })}
          >
            Related projects
          </h2>
          <div
            class={css({
              display: "grid",
              gridTemplateColumns: "repeat(3, 1fr)",
              gap: "12px",
              "@media (max-width: 640px)": { gridTemplateColumns: "1fr" },
            })}
          >
            {#each related as p (p.slug)}
              <a
                href="/projects/{p.slug}"
                onclick={() => openRelated(p)}
                class={cx(
                  "group",
                  css({
                    display: "flex",
                    flexDirection: "column",
                    rounded: "10px",
                    borderWidth: "1px",
                    borderColor: "zinc.200",
                    bg: "surface",
                    overflow: "hidden",
                    textDecoration: "none",
                    transition:
                      "border-color .18s ease, box-shadow .18s ease, transform .18s ease",
                    _hover: {
                      borderColor: "zinc.300",
                      shadow: "0 6px 20px -8px rgba(24,24,27,.12)",
                      transform: "translateY(-1px)",
                    },
                    _dark: { borderColor: "zinc.800" },
                  }),
                )}
              >
                <div
                  class={css({
                    position: "relative",
                    h: "74px",
                    bg: "surface.secondary",
                    borderBottomWidth: "1px",
                    borderColor: "zinc.100",
                    _dark: { borderColor: "zinc.800" },
                  })}
                >
                  <ProjectCover
                    seed={p.name}
                    accent={accentOf(p)}
                    cols={8}
                    rows={3}
                    cell={20}
                  />
                </div>
                <div class={css({ p: "11px 13px 13px" })}>
                  <div
                    class={css({
                      display: "flex",
                      alignItems: "baseline",
                      justifyContent: "space-between",
                      gap: "8px",
                    })}
                  >
                    <span
                      class={css({
                        fontSize: "14.5px",
                        fontWeight: "600",
                        color: "zinc.900",
                        _dark: { color: "zinc.50" },
                      })}>{p.name}</span
                    >
                    <span
                      class={css({
                        fontFamily: "geist",
                        fontSize: "10.5px",
                        color: "zinc.400",
                      })}>{formatAge(p.lastActivity)}</span
                    >
                  </div>
                  <p
                    class={css({
                      mt: "5px",
                      fontSize: "12.5px",
                      lineHeight: "1.45",
                      color: "zinc.500",
                      lineClamp: "2",
                    })}
                  >
                    {p.shortDescription}
                  </p>
                </div>
              </a>
            {/each}
          </div>
        </div>
      {/if}

      {#if prevProject || nextProject}
        <div
          class={css({
            mt: "30px",
            pt: "20px",
            borderTopWidth: "1px",
            borderColor: "zinc.100",
            display: "flex",
            justifyContent: "space-between",
            gap: "12px",
            _dark: { borderColor: "zinc.800" },
          })}
        >
          {#if prevProject}
            <a
              href="/projects/{prevProject.slug}"
              onclick={() => openRelated(prevProject!)}
              class={cx("rd-pager", pagerClass(false))}
            >
              <span class={pagerLabel}>
                <span aria-hidden="true">←</span> Previous
              </span>
              <span class={pagerName}>{prevProject.name}</span>
            </a>
          {:else}
            <span></span>
          {/if}
          {#if nextProject}
            <a
              href="/projects/{nextProject.slug}"
              onclick={() => openRelated(nextProject!)}
              class={cx("rd-pager", pagerClass(true))}
            >
              <span class={cx(pagerLabel, css({ justifyContent: "flex-end" }))}>
                Next <span aria-hidden="true">→</span>
              </span>
              <span class={cx(pagerName, css({ textAlign: "right" }))}
                >{nextProject.name}</span
              >
            </a>
          {:else}
            <span></span>
          {/if}
        </div>
      {/if}
    </div>
  </div>
</main>

<style>
  /* Two-column reading layout with a sticky meta sidebar. */
  :global(.rd-detail-grid) {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 256px;
    gap: 52px;
    margin-top: 30px;
    align-items: start;
  }
  :global(.rd-detail-side) {
    position: sticky;
    top: 28px;
    padding: 18px 20px;
    border: 1px solid #ececee;
    border-radius: 12px;
    background: #ffffffcc;
    backdrop-filter: blur(2px);
  }
  :global(.dark .rd-detail-side) {
    border-color: #27272a;
    background: #18181bcc;
  }
  :global(.rd-sidelink:hover) {
    box-shadow: 0 4px 14px -6px rgba(24, 24, 27, 0.22);
  }
  @media (max-width: 760px) {
    :global(.rd-detail-grid) {
      grid-template-columns: 1fr;
      gap: 26px;
    }
    :global(.rd-detail-side) {
      position: static;
    }
  }

  /* Code block: bordered wrapper + optional language header. Shiki paints the
     body; its own canvas/border are neutralized so the wrapper owns the chrome. */
  :global(.project-detail .rd-codeblock) {
    margin: 18px 0;
    border: 1px solid #e4e4e7;
    border-radius: 9px;
    overflow: hidden;
    background: #fafafa;
  }
  :global(.dark .project-detail .rd-codeblock) {
    border-color: #27272a;
    background: #18181b;
  }
  :global(.project-detail .rd-codeblock-head) {
    font-size: 10.5px;
    font-family: "Geist Mono", ui-monospace, monospace;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: #a1a1aa;
    padding: 8px 14px;
    border-bottom: 1px solid #ececee;
    background: #f4f4f5;
  }
  :global(.dark .project-detail .rd-codeblock-head) {
    border-bottom-color: #27272a;
    background: #27272a;
  }
  :global(.project-detail .rd-codeblock .shiki) {
    margin: 0;
    padding: 13px 14px;
    font-size: 13px;
    line-height: 1.65;
    font-family: "Geist Mono", ui-monospace, monospace;
    overflow-x: auto;
    background: transparent !important;
    scrollbar-width: thin;
    scrollbar-color: var(--code-scrollbar) transparent;
  }
  :global(.project-detail .rd-codeblock .shiki code) {
    background: none;
    border: none;
    padding: 0;
    font: inherit;
    color: inherit;
    white-space: pre;
  }
  :global(.project-detail .rd-codeblock .shiki::-webkit-scrollbar) {
    height: 0.5rem;
  }
  :global(.project-detail .rd-codeblock .shiki::-webkit-scrollbar-thumb) {
    background-color: var(--code-scrollbar);
    border-radius: 0.25rem;
  }
  /* Dark token colors only — Shiki emits them as a --shiki-dark custom property. */
  :global(.dark .project-detail .rd-codeblock .shiki),
  :global(.dark .project-detail .rd-codeblock .shiki span) {
    color: var(--shiki-dark) !important;
  }
</style>
