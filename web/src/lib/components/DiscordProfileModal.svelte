<script lang="ts">
  import { css } from "styled-system/css";
  import { flex, hstack } from "styled-system/patterns";
  import { token } from "styled-system/tokens";
  import { fade, scale } from "svelte/transition";
  import IconCopy from "~icons/material-symbols/content-copy-rounded";
  import IconCheck from "~icons/material-symbols/check-rounded";
  import { portal } from "$lib/actions/portal";

  interface Props {
    username: string;
    avatarUrl?: string;
    bannerUrl?: string;
    onclose: () => void;
  }

  let {
    username,
    avatarUrl = "https://cdn.discordapp.com/avatars/184118083143598081/798e497f55abdcadbd8440e5eed551a0.png?size=4096",
    bannerUrl = "https://cdn.discordapp.com/banners/184118083143598081/174425460b67261a124d873b016e038f.png?size=4096",
    onclose,
  }: Props = $props();

  let copySuccess = $state(false);
  let avatarFailed = $state(false);
  let bannerFailed = $state(false);

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onclose();
    }
  }

  function handleClose() {
    onclose();
  }

  async function copyUsername() {
    try {
      await navigator.clipboard.writeText(username);
      copySuccess = true;
      setTimeout(() => {
        copySuccess = false;
      }, 2000);
    } catch (err) {
      console.error("Failed to copy username:", err);
    }
  }
</script>

<div
  use:portal
  class={flex({
    align: "flex-start",
    justify: "center",
    position: "fixed",
    inset: "0",
    zIndex: "60",
    bg: "black/30",
    backdropFilter: "blur(3px)",
    p: "6",
    pt: "15vh",
  })}
  onclick={handleBackdropClick}
  onkeydown={(e) => e.key === "Escape" && handleClose()}
  role="presentation"
  tabindex="-1"
  transition:fade={{ duration: 200 }}
>
  <!-- SCALE: Adjust the scale() value to resize entire modal proportionally -->
  <div
    class={css({
      position: "relative",
      w: "full",
      maxW: "28rem",
      rounded: "xl",
      bg: "zinc.100",
      borderWidth: "1px",
      borderColor: "zinc.200",
      shadow: "lg",
      overflow: "hidden",
      transformOrigin: "top",
      sm: { scale: "1.1" },
      _dark: {
        bg: "zinc.900",
        borderColor: "zinc.700",
      },
    })}
    role="dialog"
    aria-modal="true"
    aria-labelledby="discord-profile-title"
    transition:scale={{ duration: 200, start: 0.95 }}
  >
    <!-- Banner -->
    {#if bannerUrl && !bannerFailed}
      <img
        src={bannerUrl}
        alt=""
        class={css({ h: "28", w: "full", objectFit: "cover" })}
        onerror={() => (bannerFailed = true)}
      />
    {:else}
      <div
        class={css({
          h: "28",
          backgroundImage: `linear-gradient(to bottom right, ${token.var("colors.zinc.300")}, ${token.var("colors.zinc.400")})`,
          _dark: {
            backgroundImage: `linear-gradient(to bottom right, ${token.var("colors.zinc.700")}, ${token.var("colors.zinc.800")})`,
          },
        })}
      ></div>
    {/if}

    <!-- Content area -->
    <div class={css({ px: "5", pb: "5" })}>
      <!-- Avatar with stroke effect -->
      <div
        class={css({
          position: "relative",
          mt: "-14",
          mb: "3",
          w: "fit-content",
        })}
      >
        <!-- Stroke ring (larger circle behind avatar) -->
        <!-- SIZE: avatar (96px) + stroke (4px * 2) = 104px -->
        <!-- POSITION: -m-1 centers the stroke ring behind the avatar -->
        <div
          class={css({
            position: "absolute",
            inset: "0",
            m: "-1",
            w: "104px",
            h: "104px",
            rounded: "full",
            bg: "zinc.100",
            _dark: { bg: "zinc.900" },
          })}
        ></div>

        <!-- Avatar circle -->
        <!-- SIZE: size-24 = 96px -->
        {#if avatarUrl && !avatarFailed}
          <img
            src={avatarUrl}
            alt="Profile avatar"
            class={css({
              position: "relative",
              w: "24",
              h: "24",
              rounded: "full",
              objectFit: "cover",
            })}
            onerror={() => (avatarFailed = true)}
          />
        {:else}
          <div
            class={css({
              position: "relative",
              w: "24",
              h: "24",
              rounded: "full",
              backgroundImage: `linear-gradient(to bottom right, ${token.var("colors.zinc.400")}, ${token.var("colors.zinc.500")})`,
              _dark: {
                backgroundImage: `linear-gradient(to bottom right, ${token.var("colors.zinc.500")}, ${token.var("colors.zinc.600")})`,
              },
            })}
          ></div>
        {/if}

        <!-- Online indicator -->
        <!-- POSITION: bottom/right values place center on avatar circumference -->
        <!-- For 96px avatar at 315° (bottom-right): ~4px from edge -->
        <div
          class={css({
            position: "absolute",
            bottom: "0.5",
            right: "0.5",
            w: "5",
            h: "5",
            rounded: "full",
            bg: "green.500",
            borderWidth: "3px",
            borderColor: "zinc.100",
            _dark: { borderColor: "zinc.900" },
          })}
        ></div>
      </div>

      <!-- Profile info -->
      <!-- SPACING: mb-4 controls gap before About Me section -->
      <div class={css({ mb: "4" })}>
        <h2
          id="discord-profile-title"
          class={css({
            fontSize: "xl",
            fontWeight: "bold",
            color: "zinc.900",
            _dark: { color: "zinc.100" },
          })}
        >
          Xevion
        </h2>
        <!-- USERNAME ROW: gap-1.5 controls spacing between elements -->
        <div class={hstack({ gap: "1.5", fontSize: "sm" })}>
          <span
            class={css({
              fontFamily: "mono",
              fontSize: "xs",
              px: "1.5",
              py: "0.5",
              rounded: "sm",
              borderWidth: "1px",
              borderColor: "zinc.300",
              bg: "zinc.200/50",
              color: "zinc.600",
              _dark: {
                borderColor: "zinc.700",
                bg: "zinc.800/50",
                color: "zinc.400",
              },
            })}>{username}</span
          >
          <button
            onclick={copyUsername}
            class={css({
              p: "0.5",
              rounded: "sm",
              transition: "colors",
              _hover: { bg: "zinc.200" },
              _dark: { _hover: { bg: "zinc.800" } },
            })}
            title={copySuccess ? "Copied!" : "Copy username"}
          >
            {#if copySuccess}
              <IconCheck
                class={css({
                  w: "3.5",
                  h: "3.5",
                  color: "green.600",
                  _dark: { color: "green.500" },
                })}
              />
            {:else}
              <IconCopy
                class={css({
                  w: "3.5",
                  h: "3.5",
                  color: "zinc.400",
                  _dark: { color: "zinc.500" },
                })}
              />
            {/if}
          </button>
          <span class={css({ color: "zinc.400", _dark: { color: "zinc.500" } })}
            >·</span
          >
          <span class={css({ color: "zinc.500", _dark: { color: "zinc.400" } })}
            >any/they</span
          >
        </div>
      </div>

      <!-- About Me section -->
      <div
        class={css({
          p: "3",
          rounded: "lg",
          bg: "zinc.200/50",
          borderWidth: "1px",
          borderColor: "zinc.200",
          _dark: {
            bg: "zinc.800/50",
            borderColor: "zinc.700",
          },
        })}
      >
        <h3
          class={css({
            fontSize: "xs",
            fontWeight: "semibold",
            textTransform: "uppercase",
            color: "zinc.500",
            mb: "1",
          })}
        >
          About Me
        </h3>
        <p
          class={css({
            fontSize: "sm",
            color: "zinc.700",
            _dark: { color: "zinc.300" },
          })}
        >
          Live with dignity.<br />
          <a
            href="https://xevion.dev"
            class={css({
              color: "blue.600",
              _dark: { color: "blue.400" },
              _hover: { textDecoration: "underline" },
            })}
            target="_blank"
            rel="noopener noreferrer">https://xevion.dev</a
          >
        </p>
      </div>
    </div>
  </div>
</div>
