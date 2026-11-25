<script lang="ts">
  import { enhance } from "$app/forms";
  import Button from "#lib/components/admin/Button.svelte";
  import IconTerminal from "~icons/lucide/terminal";
  import IconCheck from "~icons/lucide/circle-check";
  import { css, cx } from "styled-system/css";
  import { hstack, vstack, center } from "styled-system/patterns";
  import {
    pageTitleClass,
    pageDescriptionClass,
    iconLg,
    adminInputBase,
    errorTextClass,
    labelClass,
  } from "#lib/styles/admin.ts";
  import type { PageProps } from "./$types";

  let { data, form }: PageProps = $props();

  let working = $state(false);

  const phase = $derived(form?.outcome ?? data.request.status);

  const panelClass = css({
    rounded: "md",
    borderWidth: "1px",
    borderColor: "admin.border",
    bg: "admin.surface",
    p: "5",
    color: "admin.textSecondary",
    textWrap: "balance",
  });

  const submit = () => {
    working = true;
    return async ({
      update,
    }: {
      update: (options?: { reset?: boolean }) => Promise<void>;
    }) => {
      await update({ reset: false });
      working = false;
    };
  };
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

  {#if phase === "approved"}
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
        Approved. You can return to your terminal, the CLI is now authorized.
      </p>
    </div>
  {:else if phase === "denied"}
    <div class={panelClass}>Request denied. No token was issued.</div>
  {:else if phase === "pending" && data.request.status === "pending"}
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
        Type the verification code shown in your terminal to approve this
        request. If you did not just run the login command, deny it.
      </p>

      {#if data.request.label}
        <div class={hstack({ justify: "space-between", fontSize: "sm" })}>
          <span class={css({ color: "admin.textMuted" })}>Device</span>
          <span class={css({ color: "admin.text" })}>{data.request.label}</span>
        </div>
      {/if}

      <form
        method="POST"
        action="?/approve"
        use:enhance={submit}
        class={css({ spaceY: "3" })}
      >
        <div class={css({ spaceY: "1.5" })}>
          <label for="code" class={labelClass}>Verification code</label>
          <input
            id="code"
            name="code"
            type="text"
            autocomplete="off"
            autocapitalize="characters"
            spellcheck="false"
            required
            placeholder="XXXX-XXXX"
            class={cx(
              adminInputBase,
              css({
                fontFamily: "mono",
                fontSize: "xl",
                letterSpacing: "wider",
                textAlign: "center",
              }),
            )}
          />
          {#if form?.message}
            <p class={errorTextClass}>{form.message}</p>
          {/if}
        </div>
        <div class={hstack({ justify: "flex-end" })}>
          <Button variant="primary" type="submit" disabled={working}>
            {working ? "Working..." : "Approve"}
          </Button>
        </div>
      </form>

      <form
        method="POST"
        action="?/deny"
        use:enhance={submit}
        class={hstack({ justify: "flex-end" })}
      >
        <Button variant="ghost" type="submit" disabled={working}>
          Deny request
        </Button>
      </form>
    </div>
  {:else}
    <div class={panelClass}>
      This authorization request is invalid or has expired. Run
      <code class={css({ color: "admin.text" })}>xevion login</code> again to start
      a new one.
    </div>
  {/if}
</div>
