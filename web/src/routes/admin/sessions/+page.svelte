<script lang="ts">
  import Button from "$lib/components/admin/Button.svelte";
  import Modal from "$lib/components/admin/Modal.svelte";
  import { getSessions, revokeSession } from "$lib/api";
  import type { ApiSession } from "$lib/bindings";
  import IconMonitor from "~icons/lucide/monitor";
  import IconTerminal from "~icons/lucide/terminal";
  import IconAlert from "~icons/lucide/alert-circle";
  import { getLogger } from "@logtape/logtape";
  import { toast } from "$lib/toast";
  import { timeAgo, timeAgoExact, formatDateTime } from "$lib/time";
  import { css, cx } from "styled-system/css";
  import { hstack, vstack } from "styled-system/patterns";
  import {
    pageTitleClass,
    pageDescriptionClass,
    iconSm,
    adminCardClass,
  } from "$lib/styles/admin";

  const logger = getLogger(["admin", "sessions"]);

  let sessions = $state<ApiSession[]>([]);
  let loading = $state(true);

  let revokeModalOpen = $state(false);
  let revokeTarget = $state<ApiSession | null>(null);

  async function load() {
    loading = true;
    const result = await getSessions();
    if (result.isErr) {
      logger.error("Failed to load sessions", { error: result.error });
      toast.error(result.error.message);
      loading = false;
      return;
    }
    sessions = result.value;
    loading = false;
  }

  $effect(() => {
    load();
  });

  function initiateRevoke(session: ApiSession) {
    revokeTarget = session;
    revokeModalOpen = true;
  }

  async function confirmRevoke() {
    if (!revokeTarget) return;
    const result = await revokeSession(revokeTarget.id);
    if (result.isErr) {
      logger.error("Failed to revoke session", { error: result.error });
      toast.error(result.error.message);
      return;
    }
    toast.success("Session revoked");
    revokeModalOpen = false;
    revokeTarget = null;
    await load();
  }

  function title(session: ApiSession): string {
    if (session.sessionType === "cli") {
      return session.label ? `CLI · ${session.label}` : "CLI token";
    }
    return "Browser session";
  }
</script>

<svelte:head>
  <title>Sessions | Admin</title>
</svelte:head>

<div class={css({ maxW: "3xl", spaceY: "6" })}>
  <div>
    <h1 class={pageTitleClass}>Sessions</h1>
    <p class={pageDescriptionClass}>
      Active browser sessions and CLI tokens. Revoke any you don't recognize.
    </p>
  </div>

  {#if loading}
    <p class={css({ color: "admin.textMuted" })}>Loading…</p>
  {:else if sessions.length === 0}
    <p class={css({ color: "admin.textMuted" })}>No active sessions.</p>
  {:else}
    <div class={css({ spaceY: "3" })}>
      {#each sessions as session (session.id)}
        <div
          class={cx(
            adminCardClass,
            hstack({ justify: "space-between", gap: "4" }),
          )}
        >
          <div class={hstack({ gap: "3", alignItems: "flex-start" })}>
            <span class={css({ color: "admin.textMuted", mt: "1" })}>
              {#if session.sessionType === "cli"}
                <IconTerminal class={iconSm} />
              {:else}
                <IconMonitor class={iconSm} />
              {/if}
            </span>
            <div class={vstack({ gap: "0.5", alignItems: "flex-start" })}>
              <div class={hstack({ gap: "2" })}>
                <span
                  class={css({ fontWeight: "medium", color: "admin.text" })}
                >
                  {title(session)}
                </span>
                {#if session.current}
                  <span
                    class={css({
                      fontSize: "xs",
                      fontWeight: "medium",
                      color: "admin.accent",
                      borderWidth: "1px",
                      borderColor: "admin.accent",
                      rounded: "sm",
                      px: "1.5",
                    })}
                  >
                    this device
                  </span>
                {/if}
              </div>
              <p
                class={css({ fontSize: "sm", color: "admin.textSecondary" })}
                title={formatDateTime(session.lastActiveAt)}
              >
                Active {timeAgoExact(session.lastActiveAt)}
              </p>
              <p class={css({ fontSize: "xs", color: "admin.textMuted" })}>
                Created <span title={formatDateTime(session.createdAt)}
                  >{timeAgo(session.createdAt)}</span
                >
                · expires {formatDateTime(session.expiresAt)}
              </p>
            </div>
          </div>
          <Button variant="danger" onclick={() => initiateRevoke(session)}>
            Revoke
          </Button>
        </div>
      {/each}
    </div>
  {/if}
</div>

<Modal
  bind:open={revokeModalOpen}
  title="Revoke session"
  description={revokeTarget?.current
    ? "This is the session you're currently using. Revoking it will log you out."
    : "This immediately and permanently invalidates the session or token."}
  confirmText="Revoke"
  confirmVariant="danger"
  onconfirm={confirmRevoke}
  oncancel={() => (revokeModalOpen = false)}
>
  {#if revokeTarget}
    <div
      class={hstack({
        gap: "3",
        alignItems: "flex-start",
        rounded: "md",
        bg: "red.500/10",
        borderWidth: "1px",
        borderColor: "red.500/30",
        p: "3",
      })}
    >
      <span class={css({ color: "red.500", mt: "0.5" })}>
        <IconAlert class={iconSm} />
      </span>
      <div class={vstack({ gap: "0.5", alignItems: "flex-start" })}>
        <p class={css({ fontWeight: "medium", color: "admin.text" })}>
          {title(revokeTarget)}
        </p>
        <p
          class={css({ fontSize: "sm", color: "admin.textSecondary" })}
          title={formatDateTime(revokeTarget.lastActiveAt)}
        >
          Active {timeAgoExact(revokeTarget.lastActiveAt)}
        </p>
      </div>
    </div>
  {/if}
</Modal>
