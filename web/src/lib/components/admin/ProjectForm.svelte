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
  let description = $state("");
  let status = $state<ProjectStatus>("active");
  let githubRepo = $state("");
  let demoUrl = $state("");
  let selectedTagIds = $state<string[]>([]);
  let detailContent = $state<JSONContent | null>(null);

  // Initialize form from project prop
  $effect(() => {
    if (project) {
      name = project.name;
      slug = project.slug;
      shortDescription = project.shortDescription;
      description = project.description;
      status = project.status;
      githubRepo = project.githubRepo ?? "";
      demoUrl = project.demoUrl ?? "";
      selectedTagIds = project.tags.map((t) => t.id);
      detailContent = (project.detailContent as JSONContent | null) ?? null;
    }
  });

  let submitting = $state(false);

  // Server-returned field-level validation errors, keyed by camelCase field name.
  let fieldErrors = $state<Record<string, string>>({});

  const statusOptions = [
    { value: "active", label: "Active" },
    { value: "maintained", label: "Maintained" },
    { value: "archived", label: "Archived" },
    { value: "hidden", label: "Hidden" },
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
      description,
      status,
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

  <!-- Description -->
  <Input
    label="Description"
    type="textarea"
    bind:value={description}
    required
    rows={6}
    placeholder="A detailed description of your project..."
    help="Full project description (markdown not supported yet)"
    error={fieldErrors.description}
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
    help="Project visibility and state"
  />

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
