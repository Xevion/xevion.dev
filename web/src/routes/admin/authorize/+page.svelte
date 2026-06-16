<script lang="ts">
  import { page } from "$app/stores";
  import Button from "$lib/components/admin/Button.svelte";
  import {
    getDeviceAuthInfo,
    approveDeviceAuth,
    denyDeviceAuth,
  } from "$lib/api";
  import type { DeviceAuthInfo } from "$lib/api";
  import IconTerminal from "~icons/lucide/terminal";
  import IconCheck from "~icons/lucide/circle-check";
  import { getLogger } from "@logtape/logtape";
  import { toast } from "$lib/toast";
  import { css, cx } from "styled-system/css";
  import { hstack, vstack, center } from "styled-system/patterns";
  import {
    pageTitleClass,
    pageDescriptionClass,
    iconLg,
  } from "$lib/styles/admin";

  const logger = getLogger(["admin", "authorize"]);

  const requestId = $derived($page.url.searchParams.get("request") ?? "");
  const code = $derived($page.url.searchParams.get("code") ?? "");

  type Phase = "loading" | "pending" | "approved" | "denied" | "invalid";

  let phase = $state<Phase>("loading");
  let info = $state<DeviceAuthInfo | null>(null);
  let working = $state(false);

  $effect(() => {
    if (!requestId) {
      phase = "invalid";
      return;
    }
    void load(requestId);
  });

  async function load(id: string) {
    phase = "loading";
    const result = await getDeviceAuthInfo(id);
    if (result.isErr) {
      phase = "invalid";
      return;
    }
    info = result.value;
    phase = "pending";
  }

  async function approve() {
    working = true;
    const result = await approveDeviceAuth(
      requestId,
      code || info?.userCode || "",
    );
    working = false;
    if (result.isErr) {
      logger.error("Failed to approve device auth", { error: result.error });
      toast.error(result.error.message);
      return;
    }
    phase = "approved";
  }

  async function deny() {
    working = true;
    const result = await denyDeviceAuth(requestId);
    working = false;
    if (result.isErr) {
      logger.error("Failed to deny device auth", { error: result.error });
      toast.error(result.error.message);
      return;
    }
    phase = "denied";
  }
</script>

<svelte:head>
  <title>Authorize CLI | Admin</title>
</svelte:head>

<div class={css({ maxW: "32rem", mx: "auto", py: "8", spaceY: "6" })}>
  <div class={hstack({ gap: "4" })}>
    <span
      class={center({
        flexShrink: "0",
        w: "12",
        h: "12",
        rounded: "lg",
        bg: "admin.accent/10",
        color: "admin.accent",
      })}
    >
      <IconTerminal class={iconLg} />
    </span>
    <div>
      <h1 class={pageTitleClass}>Authorize CLI</h1>
      <p class={cx(pageDescriptionClass, css({ textWrap: "balance" }))}>
        Grant a command-line client long-lived access to your account.
      </p>
    </div>
  </div>

  {#if phase === "loading"}
    <p class={css({ color: "admin.textMuted" })}>Loading request…</p>
  {:else if phase === "invalid"}
    <div
      class={css({
        rounded: "md",
        borderWidth: "1px",
        borderColor: "admin.border",
        bg: "admin.surface",
        p: "5",
        color: "admin.textSecondary",
        textWrap: "balance",
      })}
    >
      This authorization request is invalid or has expired. Re-run
      <code class={css({ color: "admin.text" })}>xevion login</code> to start a new
      one.
    </div>
  {:else if phase === "approved"}
    <div
      class={hstack({
        gap: "3",
        alignItems: "flex-start",
        rounded: "md",
        borderWidth: "1px",
        borderColor: "green.500/30",
        bg: "green.500/10",
        p: "5",
      })}
    >
      <span class={css({ color: "green.500", flexShrink: "0", mt: "0.5" })}>
        <IconCheck class={iconLg} />
      </span>
      <p class={css({ color: "admin.text", textWrap: "balance" })}>
        Approved. You can return to your terminal — the CLI is now authorized.
      </p>
    </div>
  {:else if phase === "denied"}
    <div
      class={css({
        rounded: "md",
        borderWidth: "1px",
        borderColor: "admin.border",
        bg: "admin.surface",
        p: "5",
        color: "admin.textSecondary",
        textWrap: "balance",
      })}
    >
      Request denied. No token was issued.
    </div>
  {:else if phase === "pending" && info}
    <div
      class={vstack({
        gap: "4",
        alignItems: "stretch",
        rounded: "md",
        borderWidth: "1px",
        borderColor: "admin.border",
        bg: "admin.surface",
        p: "5",
      })}
    >
      <p
        class={css({
          color: "admin.textSecondary",
          fontSize: "sm",
          textWrap: "balance",
        })}
      >
        Confirm the code below matches the one shown in your terminal before
        approving.
      </p>

      <div class={vstack({ gap: "1", alignItems: "center", py: "2" })}>
        <span class={css({ fontSize: "xs", color: "admin.textMuted" })}>
          Verification code
        </span>
        <span
          class={css({
            fontFamily: "mono",
            fontSize: "2xl",
            fontWeight: "bold",
            letterSpacing: "wider",
            color: "admin.text",
          })}
        >
          {info.userCode}
        </span>
      </div>

      {#if info.label}
        <div class={hstack({ justify: "space-between", fontSize: "sm" })}>
          <span class={css({ color: "admin.textMuted" })}>Device</span>
          <span class={css({ color: "admin.text" })}>{info.label}</span>
        </div>
      {/if}

      <div class={hstack({ justify: "flex-end", gap: "2" })}>
        <Button variant="secondary" onclick={deny} disabled={working}>
          Deny
        </Button>
        <Button variant="primary" onclick={approve} disabled={working}>
          {working ? "Approving…" : "Approve"}
        </Button>
      </div>
    </div>
  {/if}
</div>
