<script lang="ts">
  import Button from "./Button.svelte";
  import Input from "./Input.svelte";
  import TagPicker from "./TagPicker.svelte";
  import type {
    AdminProject,
    AdminTag,
    CreateProjectData,
    ProjectStatus,
  } from "$lib/admin-types";

  interface Props {
    project?: AdminProject | null;
    availableTags: AdminTag[];
    onsubmit: (data: CreateProjectData) => Promise<void>;
    submitLabel?: string;
  }

  let {
    project = null,
    availableTags,
    onsubmit,
    submitLabel = "Save Project",
  }: Props = $props();

  // Form state
  let title = $state("");
  let slug = $state("");
  let description = $state("");
  let status = $state<ProjectStatus>("active");
  let githubRepo = $state("");
  let demoUrl = $state("");
  let icon = $state("");
  let priority = $state(0);
  let selectedTagIds = $state<string[]>([]);

  // Initialize form from project prop
  $effect(() => {
    if (project) {
      title = project.title;
      slug = project.slug;
      description = project.description;
      status = project.status;
      githubRepo = project.githubRepo ?? "";
      demoUrl = project.demoUrl ?? "";
      icon = project.icon ?? "";
      priority = project.priority;
      selectedTagIds = project.tags.map((t) => t.id);
    }
  });

  let submitting = $state(false);

  const statusOptions = [
    { value: "active", label: "Active" },
    { value: "maintained", label: "Maintained" },
    { value: "archived", label: "Archived" },
    { value: "hidden", label: "Hidden" },
  ];

  // Auto-generate slug placeholder from title
  const slugPlaceholder = $derived(
    title
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

    try {
      await onsubmit({
        title,
        slug: slug || slugPlaceholder,
        description,
        status,
        githubRepo: githubRepo || undefined,
        demoUrl: demoUrl || undefined,
        icon: icon || undefined,
        priority,
        tagIds: selectedTagIds,
      });
    } catch (error) {
      console.error("Failed to submit project:", error);
      alert("Failed to save project");
    } finally {
      submitting = false;
    }
  }
</script>

<form onsubmit={handleSubmit} class="space-y-6">
  <!-- Title & Slug -->
  <div class="grid gap-6 md:grid-cols-2">
    <Input
      label="Title"
      type="text"
      bind:value={title}
      required
      placeholder="My Awesome Project"
      help="The display name of your project"
    />

    <Input
      label="Slug"
      type="text"
      value={slug}
      oninput={handleSlugInput}
      placeholder={slugPlaceholder}
      help="URL-friendly identifier (leave empty to auto-generate)"
    />
  </div>

  <!-- Description -->
  <Input
    label="Description"
    type="textarea"
    bind:value={description}
    required
    rows={6}
    placeholder="A brief description of your project..."
    help="Plain text description (markdown not supported yet)"
  />

  <!-- Status & Priority -->
  <div class="grid gap-6 md:grid-cols-2">
    <Input
      label="Status"
      type="select"
      bind:value={status}
      options={statusOptions}
      help="Project visibility and state"
    />

    <Input
      label="Priority"
      type="number"
      bind:value={priority}
      placeholder="0"
      help="Higher numbers appear first (e.g., 100, 50, 10)"
    />
  </div>

  <!-- Links -->
  <div class="grid gap-6 md:grid-cols-2">
    <Input
      label="GitHub Repository"
      type="text"
      bind:value={githubRepo}
      placeholder="username/repo"
      help="Format: owner/repo (e.g., facebook/react)"
    />

    <Input
      label="Demo URL"
      type="url"
      bind:value={demoUrl}
      placeholder="https://example.com"
      help="Live demo or project website"
    />
  </div>

  <!-- Icon -->
  <Input
    label="Icon"
    type="text"
    bind:value={icon}
    placeholder="fa-rocket"
    help="Font Awesome icon class (e.g., fa-rocket, fa-heart)"
  />

  <!-- Tags -->
  <TagPicker
    label="Tags"
    {availableTags}
    bind:selectedTagIds
    placeholder="Search and select tags..."
  />

  <!-- Media Upload Placeholder -->
  <div class="space-y-1.5">
    <div class="block text-sm font-medium text-admin-text">Media</div>
    <Button type="button" variant="secondary" disabled class="w-full">
      <i class="fa-solid fa-upload mr-2"></i>
      Upload Images/Videos (Coming Soon)
    </Button>
    <p class="text-xs text-admin-text-muted">
      Media upload functionality will be available soon
    </p>
  </div>

  <!-- Actions -->
  <div class="flex justify-end gap-3 pt-4 border-t border-admin-border">
    <Button variant="secondary" href="/admin/projects">Cancel</Button>
    <Button type="submit" variant="primary" disabled={submitting || !title}>
      {submitting ? "Saving..." : submitLabel}
    </Button>
  </div>
</form>
