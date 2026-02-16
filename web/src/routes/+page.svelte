<script lang="ts">
  import { pushState } from "$app/navigation";
  import { page } from "$app/state";
  import ProjectCard from "$lib/components/ProjectCard.svelte";
  import DiscordProfileModal from "$lib/components/DiscordProfileModal.svelte";
  import Icon from "$lib/components/Icon.svelte";
  import { telemetry } from "$lib/telemetry";
  import type { PageData } from "./$types";
  import MaterialSymbolsVpnKey from "~icons/material-symbols/vpn-key";
  import { css, cx } from "styled-system/css";
  import { flex, wrap, grid } from "styled-system/patterns";

  let { data }: { data: PageData } = $props();
  const projects = $derived(data.projects);
  const socialLinks = $derived(data.socialLinks);

  // Filter visible social links
  const visibleSocialLinks = $derived(
    socialLinks.filter((link) => link.visible),
  );

  function openDiscordModal(username: string) {
    pushState("", { discordModal: { open: true, username } });
  }

  function trackSocialClick(url: string) {
    telemetry.trackExternalLink(url, "social");
  }

  const socialBtnClass = flex({
    align: "center",
    columnGap: "1.5",
    px: "1.5",
    py: "1",
    rounded: "sm",
    bg: "zinc.100",
    shadow: "sm",
    transition: "colors",
    cursor: "pointer",
    _dark: { bg: "zinc.900" },
    _hover: { bg: "zinc.200", _dark: { bg: "zinc.800" } },
    _focusVisible: {
      outline: "none",
      ringWidth: "2px",
      ringColor: "zinc.400",
      _dark: { ringColor: "zinc.500" },
    },
  });

  const socialIconClass = css({
    color: "zinc.600",
    _dark: { color: "zinc.300" },
  });

  const socialLabelClass = css({
    whiteSpace: "nowrap",
    fontSize: "sm",
    color: "zinc.800",
    _dark: { color: "zinc.100" },
  });
</script>

<main
  class={cx(
    "page-main",
    css({ overflowX: "hidden", fontFamily: "schibsted", pb: "12" }),
  )}
>
  <div class={flex({ direction: "column", align: "center", pt: "14" })}>
    <div
      class={css({
        maxW: "42rem",
        mx: "4",
        borderBottomWidth: "1px",
        borderColor: "zinc.200",
        divideY: "1px",
        divideColor: "zinc.200",
        _dark: { borderColor: "zinc.700", divideColor: "zinc.700" },
        sm: { mx: "6" },
      })}
    >
      <div class={flex({ direction: "column", pb: "4" })}>
        <span
          class={css({
            fontSize: "2xl",
            fontWeight: "bold",
            color: "zinc.900",
            _dark: { color: "white" },
            sm: { fontSize: "3xl" },
          })}>{data.settings.identity.displayName},</span
        >
        <span
          class={css({
            fontSize: "xl",
            fontWeight: "normal",
            color: "zinc.600",
            _dark: { color: "zinc.400" },
            sm: { fontSize: "2xl" },
          })}
        >
          {data.settings.identity.occupation}
        </span>
      </div>

      <div
        class={css({
          py: "4",
          color: "zinc.700",
          _dark: { color: "zinc.200" },
        })}
      >
        <p class={css({ sm: { fontSize: "0.95em" }, whiteSpace: "pre-line" })}>
          {data.settings.identity.bio}
        </p>
      </div>

      <div class={css({ py: "3" })}>
        <span class={css({ color: "zinc.700", _dark: { color: "zinc.200" } })}
          >Connect with me</span
        >
        <div class={wrap({ gap: "2", pl: "3", pt: "3", pb: "2" })}>
          {#each visibleSocialLinks as link (link.id)}
            {#if link.platform === "github" || link.platform === "linkedin"}
              <!-- Simple link platforms -->
              <a
                href={link.value}
                onclick={() => trackSocialClick(link.value)}
                class={socialBtnClass}
              >
                <Icon icon={link.icon} size="4" class={socialIconClass} />
                <span class={socialLabelClass}>{link.label}</span>
              </a>
            {:else if link.platform === "discord"}
              <!-- Discord - button that opens profile modal -->
              <button
                type="button"
                class={socialBtnClass}
                onclick={() => {
                  trackSocialClick(`discord:${link.value}`);
                  openDiscordModal(link.value);
                }}
              >
                <Icon icon={link.icon} size="4" class={socialIconClass} />
                <span class={socialLabelClass}>{link.label}</span>
              </button>
            {:else if link.platform === "email"}
              <!-- Email - mailto link -->
              <a
                href="mailto:{link.value}"
                onclick={() => trackSocialClick(`mailto:${link.value}`)}
                class={socialBtnClass}
              >
                <Icon icon={link.icon} size="4.5" class={socialIconClass} />
                <span class={socialLabelClass}>{link.label}</span>
              </a>
            {/if}
          {/each}
          <!-- PGP Key - links to dedicated page (tracked via page view) -->
          <a href="/pgp" class={socialBtnClass}>
            <MaterialSymbolsVpnKey
              class={css({
                w: "4.5",
                h: "4.5",
                color: "zinc.600",
                _dark: { color: "zinc.300" },
              })}
            />
            <span class={socialLabelClass}>PGP Key</span>
          </a>
        </div>
      </div>
    </div>

    <div class={css({ maxW: "42rem", mx: "4", mt: "5", sm: { mx: "6" } })}>
      <div class={grid({ columns: { base: 1, sm: 2 }, gap: "2.5" })}>
        {#each projects as project (project.id)}
          <ProjectCard {project} />
        {/each}
      </div>
    </div>
  </div>
</main>

{#if page.state.discordModal?.open}
  <DiscordProfileModal
    username={page.state.discordModal.username}
    onclose={() => history.back()}
  />
{/if}
