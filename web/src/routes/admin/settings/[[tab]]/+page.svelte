<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import Button from "$lib/components/admin/Button.svelte";
  import Input from "$lib/components/admin/Input.svelte";
  import { getSettings, updateSettings } from "$lib/api";
  import type { SiteSettings, SocialLink } from "$lib/admin-types";
  import { cn } from "$lib/utils";
  import IconGithub from "~icons/simple-icons/github";
  import IconLinkedin from "~icons/simple-icons/linkedin";
  import IconDiscord from "~icons/simple-icons/discord";
  import IconMail from "~icons/material-symbols/mail-rounded";
  import IconKey from "~icons/material-symbols/vpn-key";

  type Tab = "identity" | "social" | "admin";

  let settings = $state<SiteSettings | null>(null);
  let loading = $state(true);
  let saving = $state(false);

  // Read tab from URL, default to "identity"
  let activeTab = $derived.by(() => {
    const params = $page.params as { tab?: string };
    const tab = params.tab as Tab | undefined;
    return tab && ["identity", "social", "admin"].includes(tab)
      ? tab
      : "identity";
  });

  // Form state - will be populated when settings load
  let formData = $state<SiteSettings | null>(null);

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
    if (!formData) return;

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

  function getSocialIcon(platform: SocialLink["platform"]) {
    switch (platform) {
      case "github":
        return IconGithub;
      case "linkedin":
        return IconLinkedin;
      case "discord":
        return IconDiscord;
      case "email":
        return IconMail;
      case "pgp":
        return IconKey;
    }
  }

  function getSocialPlaceholder(platform: SocialLink["platform"]) {
    switch (platform) {
      case "github":
        return "https://github.com/username";
      case "linkedin":
        return "https://linkedin.com/in/username";
      case "discord":
        return "username";
      case "email":
        return "your.email@example.com";
      case "pgp":
        return "https://example.com/pgp-key.asc";
    }
  }

  function navigateToTab(tab: Tab) {
    // eslint-disable-next-line svelte/no-navigation-without-resolve
    goto(`/admin/settings/${tab}`, { replaceState: true });
  }
</script>

<svelte:head>
  <title>Settings | Admin</title>
</svelte:head>

<div class="space-y-6">
  <!-- Header -->
  <div>
    <h1 class="text-xl font-semibold text-zinc-50">Settings</h1>
    <p class="mt-1 text-sm text-zinc-500">
      Configure your site identity, social links, and admin preferences
    </p>
  </div>

  {#if loading}
    <div class="text-center py-12 text-zinc-500">Loading settings...</div>
  {:else if formData}
    <!-- Tabs -->
    <div class="border-b border-zinc-800">
      <nav class="flex gap-6" aria-label="Settings tabs">
        <button
          type="button"
          class={cn(
            "pb-3 px-1 text-sm font-medium border-b-2 transition-colors",
            activeTab === "identity"
              ? "border-indigo-500 text-zinc-50"
              : "border-transparent text-zinc-400 hover:text-zinc-300 hover:border-zinc-700",
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
              ? "border-indigo-500 text-zinc-50"
              : "border-transparent text-zinc-400 hover:text-zinc-300 hover:border-zinc-700",
          )}
          onclick={() => navigateToTab("social")}
        >
          Social Links
        </button>
        <button
          type="button"
          class={cn(
            "pb-3 px-1 text-sm font-medium border-b-2 transition-colors",
            activeTab === "admin"
              ? "border-indigo-500 text-zinc-50"
              : "border-transparent text-zinc-400 hover:text-zinc-300 hover:border-zinc-700",
          )}
          onclick={() => navigateToTab("admin")}
        >
          Admin Preferences
        </button>
      </nav>
    </div>

    <!-- Tab Content -->
    <div
      class="rounded-xl border border-zinc-800 bg-zinc-900 p-6 shadow-sm shadow-black/20"
    >
      {#if activeTab === "identity"}
        <div class="space-y-4">
          <h3 class="text-base font-medium text-zinc-200 mb-4">
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
            help="Supports Markdown (rendered on the index page)"
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
          <h3 class="text-base font-medium text-zinc-200 mb-4">Social Links</h3>
          <p class="text-sm text-zinc-500 mb-4">
            Configure your social media presence on the index page
          </p>

          <div class="space-y-3">
            {#each formData.socialLinks as link (link.id)}
              {@const Icon = getSocialIcon(link.platform)}
              <div
                class="rounded-lg border border-zinc-800 bg-zinc-900/50 p-4 hover:border-zinc-700 transition-colors"
              >
                <div class="flex items-start gap-4">
                  <div class="mt-2">
                    <Icon class="w-5 h-5 text-zinc-400" />
                  </div>
                  <div class="flex-1 space-y-3">
                    <div class="flex items-center justify-between">
                      <span class="text-sm font-medium text-zinc-200"
                        >{link.label}</span
                      >
                      <label class="flex items-center gap-2 cursor-pointer">
                        <span class="text-xs text-zinc-500">Visible</span>
                        <input
                          type="checkbox"
                          bind:checked={link.visible}
                          class="w-4 h-4 rounded border-zinc-700 bg-zinc-800 text-indigo-500 focus:ring-2 focus:ring-indigo-500 focus:ring-offset-0 cursor-pointer"
                        />
                      </label>
                    </div>
                    <Input
                      type={link.platform === "email" ? "email" : "text"}
                      bind:value={link.value}
                      placeholder={getSocialPlaceholder(link.platform)}
                    />
                  </div>
                </div>
              </div>
            {/each}
          </div>
        </div>
      {:else if activeTab === "admin"}
        <div class="space-y-4">
          <h3 class="text-base font-medium text-zinc-200 mb-4">
            Admin Preferences
          </h3>
          <Input
            label="Session Timeout"
            type="number"
            bind:value={formData.adminPreferences.sessionTimeoutMinutes}
            placeholder="60"
            help="Minutes of inactivity before automatic logout (5-1440)"
          />
          <Input
            label="Event Log Retention"
            type="number"
            bind:value={formData.adminPreferences.eventsRetentionDays}
            placeholder="30"
            help="Number of days to retain event logs (1-365)"
          />
          <Input
            label="Dashboard Default Tab"
            type="select"
            bind:value={formData.adminPreferences.dashboardDefaultTab}
            options={[
              { label: "Overview", value: "overview" },
              { label: "Events", value: "events" },
            ]}
            help="Which tab to show by default when visiting the dashboard"
          />
        </div>
      {/if}
    </div>

    <!-- Actions -->
    <div class="flex justify-end gap-3">
      <Button variant="secondary" onclick={handleCancel} disabled={saving}>
        Cancel
      </Button>
      <Button variant="primary" onclick={handleSave} disabled={saving}>
        {saving ? "Saving..." : "Save Changes"}
      </Button>
    </div>
  {/if}
</div>
