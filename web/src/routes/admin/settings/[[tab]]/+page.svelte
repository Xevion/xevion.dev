<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import Button from "$lib/components/admin/Button.svelte";
  import Input from "$lib/components/admin/Input.svelte";
  import { getSettings, updateSettings } from "$lib/api";
  import { getLogger } from "@logtape/logtape";
  import { toast } from "$lib/toast";
  import type { ApiSiteSettings } from "$lib/bindings";

  const logger = getLogger(["admin", "settings"]);
  import { css } from "styled-system/css";
  import { flex, hstack, grid } from "styled-system/patterns";
  import {
    pageTitleClass,
    pageDescriptionClass,
    adminCardClass,
    sectionHeadingClass,
    settingsTab,
  } from "$lib/styles/admin";

  type Tab = "identity" | "social";

  let settings = $state<ApiSiteSettings | null>(null);
  let loading = $state(true);
  let saving = $state(false);

  // Read tab from URL, default to "identity"
  let activeTab = $derived.by(() => {
    const params = $page.params as { tab?: string };
    const tab = params.tab as Tab | undefined;
    return tab && ["identity", "social"].includes(tab) ? tab : "identity";
  });

  // Form state - will be populated when settings load
  let formData = $state<ApiSiteSettings | null>(null);

  // Server-returned field-level validation errors, keyed by camelCase field name.
  let fieldErrors = $state<Record<string, string>>({});

  // Deep equality check for change detection
  const hasChanges = $derived.by(() => {
    if (!settings || !formData) return false;
    return JSON.stringify(settings) !== JSON.stringify(formData);
  });

  async function loadSettings() {
    const result = await getSettings();
    if (result.isErr) {
      logger.error("Failed to load settings", { error: result.error });
    } else {
      settings = result.value;
      formData = structuredClone(result.value);
    }
    loading = false;
  }

  $effect(() => {
    loadSettings();
  });

  async function handleSave() {
    if (!formData || !hasChanges) return;

    saving = true;
    fieldErrors = {};
    const result = await updateSettings(formData);
    if (result.isErr) {
      logger.error("Failed to save settings", { error: result.error });
      fieldErrors = result.error.fieldErrors ?? {};
      toast.error(result.error.message);
    } else {
      settings = result.value;
      formData = structuredClone(result.value);
      toast.success("Settings saved successfully!");
    }
    saving = false;
  }

  function handleCancel() {
    if (settings) {
      formData = structuredClone(settings);
    }
  }

  function navigateToTab(tab: Tab) {
    goto(`/admin/settings/${tab}`, { replaceState: true });
  }

  // Tab styling now uses shared settingsTab cva recipe from admin styles
</script>

<svelte:head>
  <title>Settings | Admin</title>
</svelte:head>

<div class={css({ spaceY: "6" })}>
  <!-- Header -->
  <div>
    <h1 class={pageTitleClass}>Settings</h1>
    <p class={pageDescriptionClass}>
      Configure your site identity and social links
    </p>
  </div>

  {#if loading}
    <div
      class={css({ textAlign: "center", py: "12", color: "admin.textMuted" })}
    >
      Loading settings...
    </div>
  {:else if formData}
    <!-- Tabs -->
    <div class={css({ borderBottomWidth: "1px", borderColor: "admin.border" })}>
      <nav class={flex({ gap: "6" })} aria-label="Settings tabs">
        <button
          type="button"
          class={settingsTab({
            state: activeTab === "identity" ? "active" : "inactive",
          })}
          onclick={() => navigateToTab("identity")}
        >
          Identity
        </button>
        <button
          type="button"
          class={settingsTab({
            state: activeTab === "social" ? "active" : "inactive",
          })}
          onclick={() => navigateToTab("social")}
        >
          Social Links
        </button>
      </nav>
    </div>

    <!-- Tab Content -->
    <div class={adminCardClass}>
      {#if activeTab === "identity"}
        <div class={css({ spaceY: "4" })}>
          <h3 class={sectionHeadingClass}>Site Identity</h3>
          <Input
            label="Display Name"
            type="text"
            bind:value={formData.identity.displayName}
            placeholder="Ryan Walters"
            required
            error={fieldErrors.displayName}
          />
          <Input
            label="Occupation/Title"
            type="text"
            bind:value={formData.identity.occupation}
            placeholder="Full-Stack Software Engineer"
            required
            error={fieldErrors.occupation}
          />
          <Input
            label="Bio/Description"
            type="textarea"
            bind:value={formData.identity.bio}
            placeholder="A brief description about yourself..."
            rows={6}
            help="Plain text for now (Markdown support coming later)"
            error={fieldErrors.bio}
          />
          <Input
            label="Site Title"
            type="text"
            bind:value={formData.identity.siteTitle}
            placeholder="Xevion.dev"
            required
            help="Displayed in browser tab and meta tags"
            error={fieldErrors.siteTitle}
          />
        </div>
      {:else if activeTab === "social"}
        <div class={css({ spaceY: "4" })}>
          <h3 class={sectionHeadingClass}>Social Links</h3>
          <p class={css({ fontSize: "sm", color: "admin.textMuted", mb: "4" })}>
            Configure your social media presence on the homepage. Display order
            and icon identifiers can be edited here.
          </p>

          <div class={css({ spaceY: "3" })}>
            {#each formData.socialLinks as link (link.id)}
              <div
                class={css({
                  rounded: "lg",
                  borderWidth: "1px",
                  borderColor: "admin.border",
                  bg: "admin.surfaceHover/50",
                  p: "4",
                  _hover: { borderColor: "admin.borderHover" },
                  transition: "colors",
                })}
              >
                <div class={flex({ align: "flex-start", gap: "4" })}>
                  <div class={css({ flex: "1", spaceY: "3" })}>
                    <div class={hstack({ justify: "space-between", gap: "0" })}>
                      <div class={hstack({ gap: "3" })}>
                        <Input
                          label="Label"
                          type="text"
                          bind:value={link.label}
                          placeholder="GitHub"
                          class={css({ w: "32" })}
                        />
                        <label
                          class={hstack({
                            gap: "2",
                            cursor: "pointer",
                            pt: "6",
                          })}
                        >
                          <span
                            class={css({
                              fontSize: "xs",
                              color: "admin.textMuted",
                            })}>Visible</span
                          >
                          <input
                            type="checkbox"
                            bind:checked={link.visible}
                            class={css({
                              w: "4",
                              h: "4",
                              rounded: "sm",
                              borderColor: "admin.border",
                              bg: "admin.bgSecondary",
                              color: "admin.accent",
                              cursor: "pointer",
                              _focus: {
                                ringWidth: "2px",
                                ringColor: "admin.accent",
                                ringOffset: "0",
                              },
                            })}
                          />
                        </label>
                      </div>
                    </div>
                    <div class={grid({ columns: 2, gap: "3" })}>
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
    <div class={flex({ justify: "flex-end", gap: "3" })}>
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
