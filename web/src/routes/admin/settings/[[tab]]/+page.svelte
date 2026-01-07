<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import Button from "$lib/components/admin/Button.svelte";
  import Input from "$lib/components/admin/Input.svelte";
  import { getSettings, updateSettings } from "$lib/api";
  import type { SiteSettings } from "$lib/admin-types";
  import { cn } from "$lib/utils";

  type Tab = "identity" | "social";

  let settings = $state<SiteSettings | null>(null);
  let loading = $state(true);
  let saving = $state(false);

  // Read tab from URL, default to "identity"
  let activeTab = $derived.by(() => {
    const params = $page.params as { tab?: string };
    const tab = params.tab as Tab | undefined;
    return tab && ["identity", "social"].includes(tab) ? tab : "identity";
  });

  // Form state - will be populated when settings load
  let formData = $state<SiteSettings | null>(null);

  // Deep equality check for change detection
  const hasChanges = $derived.by(() => {
    if (!settings || !formData) return false;
    return JSON.stringify(settings) !== JSON.stringify(formData);
  });

  async function loadSettings() {
    try {
      const data = await getSettings();
      settings = data;
      formData = structuredClone(data);
    } catch (error) {
      console.error("Failed to load settings:", error);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    loadSettings();
  });

  async function handleSave() {
    if (!formData || !hasChanges) return;

    saving = true;
    try {
      const updated = await updateSettings(formData);
      settings = updated;
      formData = structuredClone(updated);
      alert("Settings saved successfully!");
    } catch (error) {
      console.error("Failed to save settings:", error);
      alert("Failed to save settings");
    } finally {
      saving = false;
    }
  }

  function handleCancel() {
    if (settings) {
      formData = structuredClone(settings);
    }
  }

  function navigateToTab(tab: Tab) {
    goto(`/admin/settings/${tab}`, { replaceState: true });
  }
</script>

<svelte:head>
  <title>Settings | Admin</title>
</svelte:head>

<div class="space-y-6">
  <!-- Header -->
  <div>
    <h1 class="text-xl font-semibold text-admin-text">Settings</h1>
    <p class="mt-1 text-sm text-admin-text-muted">
      Configure your site identity and social links
    </p>
  </div>

  {#if loading}
    <div class="text-center py-12 text-admin-text-muted">
      Loading settings...
    </div>
  {:else if formData}
    <!-- Tabs -->
    <div class="border-b border-admin-border">
      <nav class="flex gap-6" aria-label="Settings tabs">
        <button
          type="button"
          class={cn(
            "pb-3 px-1 text-sm font-medium border-b-2 transition-colors",
            activeTab === "identity"
              ? "border-admin-accent text-admin-text"
              : "border-transparent text-admin-text-muted hover:text-admin-text hover:border-admin-border-hover",
          )}
          onclick={() => navigateToTab("identity")}
        >
          Identity
        </button>
        <button
          type="button"
          class={cn(
            "pb-3 px-1 text-sm font-medium border-b-2 transition-colors",
            activeTab === "social"
              ? "border-admin-accent text-admin-text"
              : "border-transparent text-admin-text-muted hover:text-admin-text hover:border-admin-border-hover",
          )}
          onclick={() => navigateToTab("social")}
        >
          Social Links
        </button>
      </nav>
    </div>

    <!-- Tab Content -->
    <div
      class="rounded-xl border border-admin-border bg-admin-surface p-6 shadow-sm shadow-black/10 dark:shadow-black/20"
    >
      {#if activeTab === "identity"}
        <div class="space-y-4">
          <h3 class="text-base font-medium text-admin-text mb-4">
            Site Identity
          </h3>
          <Input
            label="Display Name"
            type="text"
            bind:value={formData.identity.displayName}
            placeholder="Ryan Walters"
            required
          />
          <Input
            label="Occupation/Title"
            type="text"
            bind:value={formData.identity.occupation}
            placeholder="Full-Stack Software Engineer"
            required
          />
          <Input
            label="Bio/Description"
            type="textarea"
            bind:value={formData.identity.bio}
            placeholder="A brief description about yourself..."
            rows={6}
            help="Plain text for now (Markdown support coming later)"
          />
          <Input
            label="Site Title"
            type="text"
            bind:value={formData.identity.siteTitle}
            placeholder="Xevion.dev"
            required
            help="Displayed in browser tab and meta tags"
          />
        </div>
      {:else if activeTab === "social"}
        <div class="space-y-4">
          <h3 class="text-base font-medium text-admin-text mb-4">
            Social Links
          </h3>
          <p class="text-sm text-admin-text-muted mb-4">
            Configure your social media presence on the homepage. Display order
            and icon identifiers can be edited here.
          </p>

          <div class="space-y-3">
            {#each formData.socialLinks as link (link.id)}
              <div
                class="rounded-lg border border-admin-border bg-admin-surface-hover/50 p-4 hover:border-admin-border-hover transition-colors"
              >
                <div class="flex items-start gap-4">
                  <div class="flex-1 space-y-3">
                    <div class="flex items-center justify-between">
                      <div class="flex items-center gap-3">
                        <Input
                          label="Label"
                          type="text"
                          bind:value={link.label}
                          placeholder="GitHub"
                          class="w-32"
                        />
                        <label
                          class="flex items-center gap-2 cursor-pointer pt-6"
                        >
                          <span class="text-xs text-admin-text-muted"
                            >Visible</span
                          >
                          <input
                            type="checkbox"
                            bind:checked={link.visible}
                            class="w-4 h-4 rounded border-admin-border bg-admin-bg-secondary text-admin-accent focus:ring-2 focus:ring-admin-accent focus:ring-offset-0 cursor-pointer"
                          />
                        </label>
                      </div>
                    </div>
                    <div class="grid grid-cols-2 gap-3">
                      <Input
                        label="Platform"
                        type="text"
                        bind:value={link.platform}
                        placeholder="github"
                        help="Platform identifier (github, linkedin, discord, email, etc.)"
                      />
                      <Input
                        label="Display Order"
                        type="number"
                        bind:value={link.displayOrder}
                        placeholder="1"
                        help="Lower numbers appear first"
                      />
                    </div>
                    <Input
                      label="Icon"
                      type="text"
                      bind:value={link.icon}
                      placeholder="simple-icons:github"
                      help="Icon identifier (e.g., 'simple-icons:github', 'lucide:mail')"
                    />
                    <Input
                      label="Value"
                      type={link.platform === "email" ? "email" : "text"}
                      bind:value={link.value}
                      placeholder={link.platform === "github"
                        ? "https://github.com/username"
                        : link.platform === "email"
                          ? "your.email@example.com"
                          : "value"}
                      help={link.platform === "discord"
                        ? "Discord username (copied to clipboard on click)"
                        : link.platform === "email"
                          ? "Email address (opens mailto: link)"
                          : "URL to link to"}
                    />
                  </div>
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </div>

    <!-- Actions -->
    <div class="flex justify-end gap-3">
      <Button
        variant="secondary"
        onclick={handleCancel}
        disabled={!hasChanges || saving}
      >
        Cancel
      </Button>
      <Button
        variant="primary"
        onclick={handleSave}
        disabled={!hasChanges || saving}
      >
        {saving ? "Saving..." : "Save Changes"}
      </Button>
    </div>
  {/if}
</div>
