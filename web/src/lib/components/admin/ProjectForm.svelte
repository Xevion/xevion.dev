<script lang="ts">
  import { css } from "styled-system/css";
  import { flex, grid } from "styled-system/patterns";
  import Button from "./Button.svelte";
  import Input from "./Input.svelte";
  import TagPicker from "./TagPicker.svelte";
  import MediaManager from "./MediaManager.svelte";
  import ContentEditor from "./ContentEditor.svelte";
  import type {
    ApiAdminProject,
    ApiProjectDetail,
    ProjectStatus,
  } from "$lib/bindings";
  import type { JSONContent } from "@tiptap/core";
  import type { CreateProjectData, TagWithIcon } from "$lib/admin-types";
  import type { ApiError } from "$lib/errors";
  import type { Result } from "true-myth/result";
  import { getLogger } from "@logtape/logtape";
  import { toast } from "$lib/toast";

  const logger = getLogger(["admin", "components", "ProjectForm"]);

  interface Props {
    project?: ApiProjectDetail | null;
    availableTags: TagWithIcon[];
    onsubmit: (
      data: CreateProjectData,
    ) => Promise<Result<ApiAdminProject, ApiError>>;
    ondelete?: () => void;
    submitLabel?: string;
  }

  let {
    project = null,
    availableTags,
    onsubmit,
    ondelete,
    submitLabel = "Save Project",
  }: Props = $props();

  // Form state
  let name = $state("");
  let slug = $state("");
  let shortDescription = $state("");
  let status = $state<ProjectStatus>("active");
  // Overall public visibility, independent of activity status.
  let hidden = $state(false);
  // Source is private: hides repo links but keeps GitHub activity syncing.
  // Named `isPrivate` because `private` is a reserved word for a local binding.
  let isPrivate = $state(false);
  let githubRepo = $state("");
  let demoUrl = $state("");
  let selectedTagIds = $state<string[]>([]);
  // Initialized synchronously (not in the $effect below) because ContentEditor
  // snapshots this value in its onMount to build the editor — an $effect would
  // run after the child mounts, leaving the editor empty on edit. Reading the
  // initial `project` value is correct because the edit route keys the form on
  // project.id, so each project gets a fresh mount (and thus a fresh snapshot).
  // svelte-ignore state_referenced_locally
  let detailContent = $state<JSONContent | null>(
    (project?.detailContent as JSONContent | null) ?? null,
  );

  // Initialize form from project prop
  $effect(() => {
    if (project) {
      name = project.name;
      slug = project.slug;
      shortDescription = project.shortDescription;
      status = project.status;
      hidden = project.hidden;
      isPrivate = project.private;
      githubRepo = project.githubRepo ?? "";
      demoUrl = project.demoUrl ?? "";
      selectedTagIds = project.tags.map((t) => t.id);
    }
  });

  let submitting = $state(false);

  // Server-returned field-level validation errors, keyed by camelCase field name.
  let fieldErrors = $state<Record<string, string>>({});

  const statusOptions = [
    { value: "active", label: "Active" },
    { value: "maintained", label: "Maintained" },
    { value: "archived", label: "Archived" },
  ];

  // Auto-generate slug placeholder from name
  const slugPlaceholder = $derived(
    name
      .toLowerCase()
      .replace(/[^\w\s-]/g, "")
      .replace(/[\s_-]+/g, "-")
      .replace(/^-+|-+$/g, ""),
  );

  function handleSlugInput(value: string | number) {
    slug = value as string;
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    submitting = true;
    fieldErrors = {};

    const result = await onsubmit({
      name,
      slug: slug || slugPlaceholder,
      shortDescription,
      status,
      hidden,
      private: isPrivate,
      githubRepo: githubRepo || undefined,
      demoUrl: demoUrl || undefined,
      tagIds: selectedTagIds,
      detailContent: detailContent ?? undefined,
    });

    if (result.isErr) {
      logger.error("Failed to submit project", { error: result.error });
      fieldErrors = result.error.fieldErrors ?? {};
      toast.error(result.error.message);
    }
    submitting = false;
  }
</script>

<form onsubmit={handleSubmit} class={css({ spaceY: "6" })}>
  <!-- Title & Slug -->
  <div class={grid({ columns: { md: 2 }, gap: "6" })}>
    <Input
      label="Name"
      type="text"
      bind:value={name}
      required
      placeholder="My Awesome Project"
      help="The display name of your project"
      error={fieldErrors.name}
    />

    <Input
      label="Slug"
      type="text"
      value={slug}
      oninput={handleSlugInput}
      placeholder={slugPlaceholder}
      help="URL-friendly identifier (leave empty to auto-generate)"
      error={fieldErrors.slug}
      inputClass={css({ fontFamily: "geist" })}
    />
  </div>

  <!-- Short Description -->
  <Input
    label="Short Description"
    type="text"
    bind:value={shortDescription}
    required
    placeholder="A concise one-line summary"
    help="Brief description shown in project cards"
    error={fieldErrors.shortDescription}
  />

  <!-- Detail Content -->
  <ContentEditor
    label="Detail Content"
    help="Rich content for the project's /projects/{slug} page. Leave empty for no detail page (card links straight to demo/GitHub)."
    bind:content={detailContent}
  />

  <!-- Status -->
  <Input
    label="Status"
    type="select"
    bind:value={status}
    options={statusOptions}
    help="Activity/development state (separate from visibility)"
  />

  <!-- Visibility & source -->
  <div class={css({ display: "flex", flexDirection: "column", gap: "3" })}>
    <label
      class={css({
        display: "flex",
        alignItems: "flex-start",
        gap: "2",
        cursor: "pointer",
      })}
    >
      <input
        type="checkbox"
        bind:checked={hidden}
        class={css({ mt: "1", cursor: "pointer" })}
      />
      <span>
        <span class={css({ fontSize: "sm", fontWeight: "medium" })}>Hidden</span
        >
        <span
          class={css({
            display: "block",
            fontSize: "xs",
            color: "admin.textMuted",
          })}
        >
          Hide from all public listings and the project's public page.
        </span>
      </span>
    </label>

    <label
      class={css({
        display: "flex",
        alignItems: "flex-start",
        gap: "2",
        cursor: "pointer",
      })}
    >
      <input
        type="checkbox"
        bind:checked={isPrivate}
        class={css({ mt: "1", cursor: "pointer" })}
      />
      <span>
        <span class={css({ fontSize: "sm", fontWeight: "medium" })}
          >Private</span
        >
        <span
          class={css({
            display: "block",
            fontSize: "xs",
            color: "admin.textMuted",
          })}
        >
          Source is private: hides the GitHub link, but the repo is still synced
          for activity.
        </span>
      </span>
    </label>
  </div>

  <!-- Links -->
  <div class={grid({ columns: { md: 2 }, gap: "6" })}>
    <Input
      label="GitHub Repository"
      type="text"
      bind:value={githubRepo}
      placeholder="username/repo"
      help="Format: owner/repo (e.g., facebook/react)"
      error={fieldErrors.githubRepo}
    />

    <Input
      label="Demo URL"
      type="url"
      bind:value={demoUrl}
      placeholder="https://example.com"
      help="Live demo or project website"
      error={fieldErrors.demoUrl}
    />
  </div>

  <!-- Tags -->
  <TagPicker
    label="Tags"
    {availableTags}
    bind:selectedTagIds
    placeholder="Search and select tags..."
  />

  <!-- Media -->
  <MediaManager projectId={project?.id ?? null} media={project?.media ?? []} />

  <!-- Actions -->
  <div
    class={flex({
      justify: "space-between",
      gap: "3",
      pt: "4",
      borderTopWidth: "1px",
      borderColor: "admin.border",
    })}
  >
    {#if ondelete}
      <Button variant="danger" onclick={ondelete}>Delete</Button>
    {:else}
      <div></div>
    {/if}
    <div class={flex({ gap: "3" })}>
      <Button variant="secondary" href="/admin/projects">Cancel</Button>
      <Button type="submit" variant="primary" disabled={submitting || !name}>
        {submitting ? "Saving..." : submitLabel}
      </Button>
    </div>
  </div>
</form>
