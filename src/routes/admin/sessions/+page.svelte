<script lang="ts">
  import Button from "#lib/components/admin/Button.svelte";
  import Modal from "#lib/components/admin/Modal.svelte";
  import { enhance } from "$app/forms";
  import type { ApiSession } from "#lib/bindings/index.ts";
  import IconMonitor from "~icons/lucide/monitor";
  import IconTerminal from "~icons/lucide/terminal";
  import IconAlert from "~icons/lucide/alert-circle";
  import { toast } from "#lib/toast.ts";
  import type { PageProps } from "./$types";
  import { timeAgo, timeAgoExact, formatDateTime } from "#lib/time.ts";
  import { css, cx } from "styled-system/css";
  import { hstack, vstack } from "styled-system/patterns";
  import {
    pageTitleClass,
    pageDescriptionClass,
    iconSm,
    adminCardClass,
  } from "#lib/styles/admin.ts";

  let { data }: PageProps = $props();

  let revokeModalOpen = $state(false);
  let revokeTarget = $state<ApiSession | null>(null);
  let revokeForm = $state<HTMLFormElement>();
  const sessions = $derived(data.sessions);

  function initiateRevoke(session: ApiSession) {
    revokeTarget = session;
    revokeModalOpen = true;
  }

  function confirmRevoke() {
    revokeForm?.requestSubmit();
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

  {#if sessions.length === 0}
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

<form
  method="POST"
  action="?/revoke"
  bind:this={revokeForm}
  use:enhance={() =>
    async ({ result, update }) => {
      await update();
      if (result.type === "success") toast.success("Session revoked");
      else if (result.type === "failure")
        toast.error(String(result.data?.message ?? "Failed to revoke session"));
      revokeTarget = null;
    }}
>
  <input type="hidden" name="id" value={revokeTarget?.id ?? ""} />
</form>

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
