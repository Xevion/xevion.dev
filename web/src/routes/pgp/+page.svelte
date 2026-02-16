<script lang="ts">
  import { OverlayScrollbarsComponent } from "overlayscrollbars-svelte";
  import { telemetry } from "$lib/telemetry";
  import IconDownload from "~icons/material-symbols/download-rounded";
  import IconCopy from "~icons/material-symbols/content-copy-rounded";
  import IconCheck from "~icons/material-symbols/check-rounded";
  import type { PageData } from "./$types";
  import { css, cx } from "styled-system/css";
  import { flex, center } from "styled-system/patterns";

  let { data }: { data: PageData } = $props();

  let copySuccess = $state(false);
  let copyCommandSuccess = $state(false);

  async function copyToClipboard() {
    telemetry.track({
      name: "pgp_interaction",
      properties: { action: "copy_key" },
    });
    try {
      await navigator.clipboard.writeText(data.key.content);
      copySuccess = true;
      setTimeout(() => {
        copySuccess = false;
      }, 2000);
    } catch (err) {
      console.error("Failed to copy:", err);
    }
  }

  async function copyCommand() {
    telemetry.track({
      name: "pgp_interaction",
      properties: { action: "copy_command" },
    });
    try {
      await navigator.clipboard.writeText(
        "curl https://xevion.dev/pgp | gpg --import",
      );
      copyCommandSuccess = true;
      setTimeout(() => {
        copyCommandSuccess = false;
      }, 2000);
    } catch (err) {
      console.error("Failed to copy command:", err);
    }
  }

  function downloadKey() {
    telemetry.track({
      name: "pgp_interaction",
      properties: { action: "download_key" },
    });
    const a = document.createElement("a");
    a.href = "/publickey.asc";
    a.download = "publickey.asc";
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
  }
</script>

<svelte:head>
  <title>PGP Public Key - Ryan Walters</title>
  <meta
    name="description"
    content="Download or copy Ryan Walters' PGP public key"
  />
</svelte:head>

<main
  class={cx("page-main", css({ overflowX: "hidden", fontFamily: "schibsted" }))}
>
  <div
    class={flex({
      direction: "column",
      align: "center",
      pt: "14",
      pb: "20",
      px: "4",
      sm: { px: "6" },
    })}
  >
    <div class={css({ maxW: "42rem", w: "full" })}>
      <!-- Header -->
      <div class={css({ mb: "6" })}>
        <h1
          class={css({
            fontSize: "2xl",
            fontWeight: "bold",
            color: "zinc.900",
            mb: "2",
            _dark: { color: "white" },
            sm: { fontSize: "3xl" },
          })}
        >
          PGP Public Key
        </h1>
        <p
          class={css({
            fontSize: "sm",
            color: "zinc.600",
            _dark: { color: "zinc.400" },
            sm: { fontSize: "base" },
          })}
        >
          Use this key to send me encrypted messages or verify my signed
          content.
        </p>
      </div>

      <!-- Fingerprint -->
      <div
        class={css({
          mb: "6",
          p: "3",
          bg: "zinc.100",
          rounded: "lg",
          borderWidth: "1px",
          borderColor: "zinc.200",
          _dark: { bg: "zinc.800", borderColor: "zinc.700" },
          sm: { p: "4" },
        })}
      >
        <div
          class={css({
            fontSize: "xs",
            fontWeight: "semibold",
            color: "zinc.700",
            mb: "2",
            _dark: { color: "zinc.300" },
            sm: { fontSize: "sm" },
          })}
        >
          Key Fingerprint
        </div>
        <div
          class={css({
            fontFamily: "mono",
            fontSize: "sm",
            color: "zinc.900",
            wordBreak: "break-all",
            _dark: { color: "zinc.100" },
            sm: { fontSize: "base" },
          })}
        >
          {data.key.fingerprint}
        </div>
        <div
          class={css({
            mt: "3",
            pt: "3",
            borderTopWidth: "1px",
            borderColor: "zinc.200",
            spaceY: "1",
            _dark: { borderColor: "zinc.700" },
          })}
        >
          <div
            class={css({
              fontSize: "xs",
              color: "zinc.600",
              _dark: { color: "zinc.400" },
              sm: { fontSize: "sm" },
            })}
          >
            <span class={css({ fontWeight: "medium" })}>Key ID:</span>
            <span class={css({ fontFamily: "mono", ml: "2" })}
              >{data.key.keyId}</span
            >
          </div>
          <div
            class={css({
              fontSize: "xs",
              color: "zinc.600",
              _dark: { color: "zinc.400" },
              sm: { fontSize: "sm" },
            })}
          >
            <span class={css({ fontWeight: "medium" })}>Email:</span>
            <span class={css({ ml: "2" })}>{data.key.email}</span>
          </div>
        </div>
      </div>

      <!-- Key Content Card -->
      <div
        class={css({
          mb: "6",
          borderWidth: "1px",
          borderColor: "zinc.200",
          rounded: "lg",
          overflow: "hidden",
          bg: "white",
          _dark: { borderColor: "zinc.700", bg: "zinc.900" },
        })}
      >
        <div
          class={css({
            px: "3",
            py: "2",
            bg: "zinc.50",
            borderBottomWidth: "1px",
            borderColor: "zinc.200",
            _dark: { bg: "zinc.800", borderColor: "zinc.700" },
            sm: { px: "4", py: "3" },
          })}
        >
          <div
            class={css({
              fontSize: "xs",
              fontWeight: "semibold",
              color: "zinc.700",
              _dark: { color: "zinc.300" },
              sm: { fontSize: "sm" },
            })}
          >
            Public Key
          </div>
        </div>
        <OverlayScrollbarsComponent
          options={{
            scrollbars: { autoHide: "leave", autoHideDelay: 800 },
          }}
          defer
          style="max-height: 400px"
        >
          <pre
            class={css({
              p: "3",
              fontSize: "xs",
              fontFamily: "mono",
              color: "zinc.800",
              bg: "zinc.50",
              overflowX: "auto",
              _dark: { color: "zinc.200", bg: "zinc.900/50" },
              sm: { p: "4" },
            })}>{data.key.content}</pre>
        </OverlayScrollbarsComponent>
      </div>

      <!-- Action Buttons -->
      <div
        class={flex({
          direction: "column",
          gap: "2",
          sm: { flexDirection: "row", gap: "3" },
        })}
      >
        <button
          onclick={copyToClipboard}
          class={center({
            gap: "2",
            px: "3",
            py: "2",
            rounded: "sm",
            bg: "zinc.900",
            color: "white",
            shadow: "sm",
            transition: "colors",
            cursor: "pointer",
            _dark: { bg: "zinc.100", color: "zinc.900" },
            _hover: { bg: "zinc.800", _dark: { bg: "zinc.200" } },
            sm: { px: "4", py: "2.5" },
          })}
        >
          <IconCopy class={css({ w: "4", h: "4", sm: { w: "5", h: "5" } })} />
          <span
            class={css({
              fontSize: "sm",
              fontWeight: "medium",
              sm: { fontSize: "base" },
            })}>{copySuccess ? "Copied!" : "Copy to Clipboard"}</span
          >
        </button>
        <button
          onclick={downloadKey}
          class={center({
            gap: "2",
            px: "3",
            py: "2",
            rounded: "sm",
            bg: "zinc.100",
            color: "zinc.800",
            transition: "colors",
            cursor: "pointer",
            _dark: { bg: "zinc.800", color: "zinc.100" },
            _hover: { bg: "zinc.200", _dark: { bg: "zinc.700" } },
            sm: { px: "4", py: "2.5" },
          })}
        >
          <IconDownload
            class={css({ w: "4", h: "4", sm: { w: "5", h: "5" } })}
          />
          <span
            class={css({
              fontSize: "sm",
              fontWeight: "medium",
              sm: { fontSize: "base" },
            })}>Download</span
          >
        </button>
      </div>

      <!-- Additional Info -->
      <div
        class={css({
          mt: "8",
          p: "3",
          bg: "zinc.50",
          rounded: "lg",
          borderWidth: "1px",
          borderColor: "zinc.200",
          _dark: { bg: "zinc.800/50", borderColor: "zinc.700" },
          sm: { p: "4" },
        })}
      >
        <h2
          class={css({
            fontSize: "xs",
            fontWeight: "semibold",
            color: "zinc.700",
            mb: "2",
            _dark: { color: "zinc.300" },
            sm: { fontSize: "sm" },
          })}
        >
          How to use this key
        </h2>
        <div
          class={css({
            fontSize: "xs",
            color: "zinc.600",
            spaceY: "2",
            _dark: { color: "zinc.400" },
            sm: { fontSize: "sm" },
          })}
        >
          <p>
            Import this key into your GPG keyring to encrypt messages for me or
            verify my signatures:
          </p>
          <div class={css({ position: "relative" })}>
            <pre
              class={css({
                p: "2",
                pr: "12",
                bg: "white",
                rounded: "sm",
                borderWidth: "1px",
                borderColor: "zinc.200",
                fontFamily: "mono",
                fontSize: "xs",
                overflowX: "auto",
                _dark: { bg: "zinc.900", borderColor: "zinc.700" },
                sm: { p: "3" },
              })}>curl https://xevion.dev/pgp | gpg --import</pre>
            <button
              onclick={copyCommand}
              disabled={copyCommandSuccess}
              class={cx(
                css({
                  position: "absolute",
                  top: "50%",
                  transform: "translateY(-50%)",
                  right: "2",
                  p: "1",
                  rounded: "sm",
                  borderWidth: "1px",
                  borderColor: "zinc.300",
                  bg: "zinc.50",
                  transition: "all",
                  _dark: { borderColor: "zinc.600", bg: "zinc.800" },
                  _hover: {
                    bg: "zinc.100",
                    borderColor: "zinc.400",
                    _dark: { bg: "zinc.700", borderColor: "zinc.500" },
                  },
                }),
                copyCommandSuccess
                  ? css({ cursor: "default" })
                  : css({ cursor: "pointer" }),
              )}
              title={copyCommandSuccess ? "Copied!" : "Copy command"}
            >
              {#if copyCommandSuccess}
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
                    color: "zinc.600",
                    _dark: { color: "zinc.400" },
                  })}
                />
              {/if}
            </button>
          </div>
          <p
            class={css({
              fontSize: "xs",
              color: "zinc.500",
              _dark: { color: "zinc.500" },
            })}
          >
            You can also find this key on public keyservers by searching for the
            fingerprint above.
          </p>
        </div>
      </div>
    </div>
  </div>
</main>
