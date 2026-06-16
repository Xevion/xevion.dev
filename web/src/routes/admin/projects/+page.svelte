<script lang="ts">
  import Button from "$lib/components/admin/Button.svelte";
  import Table from "$lib/components/admin/Table.svelte";
  import TagChip from "$lib/components/TagChip.svelte";
  import { goto } from "$app/navigation";
  import type { PageData } from "./$types";
  import type { ProjectStatus } from "$lib/bindings";
  import IconPlus from "~icons/lucide/plus";
  import IconAlert from "~icons/lucide/alert-circle";
  import { css, cx } from "styled-system/css";
  import { hstack, wrap } from "styled-system/patterns";
  import {
    pageTitleClass,
    pageDescriptionClass,
    iconSm,
  } from "$lib/styles/admin";
  import { timeAgo, formatDateTime } from "$lib/time";

  // Status display configuration (colors match Badge component)
  const STATUS_CONFIG: Record<ProjectStatus, { color: string; label: string }> =
    {
      active: { color: "10b981", label: "Active" },
      maintained: { color: "6366f1", label: "Maintained" },
      archived: { color: "71717a", label: "Archived" },
      hidden: { color: "52525b", label: "Hidden" },
    };

  let { data }: { data: PageData } = $props();

  const thClass = css({
    px: "4",
    py: "3",
    textAlign: "left",
    fontSize: "xs",
    fontWeight: "medium",
    color: "admin.textMuted",
  });
</script>

<svelte:head>
  <title>Projects | Admin</title>
</svelte:head>

<div class={css({ spaceY: "6" })}>
  <!-- Header -->
  <div class={hstack({ justify: "space-between", gap: "0" })}>
    <div>
      <h1 class={pageTitleClass}>Projects</h1>
      <p class={pageDescriptionClass}>Manage your project portfolio</p>
    </div>
    <Button variant="primary" href="/admin/projects/new">
      <IconPlus class={cx(iconSm, css({ mr: "2" }))} />
      New Project
    </Button>
  </div>

  <!-- Projects Table -->
  {#if data.projects.length === 0}
    <div class={css({ textAlign: "center", py: "12" })}>
      <p class={css({ color: "admin.textMuted", mb: "4" })}>No projects yet</p>
      <Button variant="primary" href="/admin/projects/new"
        >Create your first project</Button
      >
    </div>
  {:else}
    <Table>
      <thead class={css({ bg: "admin.surfaceHover" })}>
        <tr>
          <th class={thClass}> Name </th>
          <th class={thClass}> Status </th>
          <th class={thClass}> Tags </th>
          <th class={thClass}> Last Activity </th>
        </tr>
      </thead>
      <tbody class={css({ divideY: "1px", divideColor: "admin.border" })}>
        {#each data.projects as project (project.id)}
          <tr
            class={css({
              _hover: { bg: "admin.surfaceHover/50" },
              transition: "colors",
              cursor: "pointer",
            })}
            onclick={() => goto(`/admin/projects/${project.id}`)}
            onkeydown={(e) =>
              (e.key === "Enter" || e.key === " ") &&
              goto(`/admin/projects/${project.id}`)}
            role="link"
            tabindex="0"
          >
            <td class={css({ px: "4", py: "3" })}>
              <div class={hstack({ gap: "3" })}>
                <div>
                  <div
                    class={css({ fontWeight: "medium", color: "admin.text" })}
                  >
                    {project.name}
                  </div>
                  <div
                    class={css({ fontSize: "xs", color: "admin.textMuted" })}
                  >
                    {project.slug}
                  </div>
                </div>
              </div>
            </td>
            <td class={css({ px: "4", py: "3" })}>
              <TagChip
                name={STATUS_CONFIG[project.status].label}
                color={STATUS_CONFIG[project.status].color}
              />
            </td>
            <td class={css({ px: "4", py: "3" })}>
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div
                class={wrap({ gap: "1" })}
                onclick={(e) => e.stopPropagation()}
                onkeydown={(e) => e.stopPropagation()}
              >
                {#each project.tags.slice(0, 3) as tag (tag.id)}
                  <TagChip
                    name={tag.name}
                    color={tag.color}
                    icon={tag.icon}
                    href={`/admin/tags/${tag.slug}`}
                  />
                {/each}
                {#if project.tags.length > 3}
                  <span
                    class={css({
                      display: "inline-flex",
                      alignItems: "center",
                      px: "2",
                      py: "1",
                      fontSize: "xs",
                      color: "admin.textMuted",
                      bg: "admin.surfaceHover",
                      rounded: "sm",
                    })}
                  >
                    +{project.tags.length - 3}
                  </span>
                {/if}
              </div>
            </td>
            <td
              class={css({
                px: "4",
                py: "3",
                color: "admin.textSecondary",
                fontSize: "sm",
              })}
            >
              <span class={hstack({ gap: "1.5", alignItems: "center" })}>
                <span
                  title={project.githubSyncedAt
                    ? `Last activity: ${formatDateTime(project.lastActivity)}\nLast synced: ${formatDateTime(project.githubSyncedAt)}`
                    : formatDateTime(project.lastActivity)}
                >
                  {timeAgo(project.lastActivity)}
                </span>
                {#if project.githubSyncError}
                  <span
                    class={css({ display: "inline-flex", color: "#f59e0b" })}
                    title={`GitHub sync error: ${project.githubSyncError}`}
                  >
                    <IconAlert class={iconSm} aria-label="GitHub sync error" />
                  </span>
                {/if}
              </span>
            </td>
          </tr>
        {/each}
      </tbody>
    </Table>
  {/if}
</div>
