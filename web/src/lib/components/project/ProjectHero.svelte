<script lang="ts">
  import { css } from "styled-system/css";
  import type { ApiProjectDetail } from "$lib/bindings";
  import TerminalCast from "./TerminalCast.svelte";
  import ClosedSourceCallout from "./ClosedSourceCallout.svelte";

  // Adaptive hero, chosen in priority order:
  //   1. terminalCast present  → terminal cast (CLI archetype)
  //   2. else private          → closed-source callout
  //   3. else                  → nothing (visual/web projects go straight to body)
  interface Props {
    project: ApiProjectDetail;
  }

  let { project }: Props = $props();
</script>

{#if project.terminalCast}
  <div class={css({ mt: "22px" })}>
    <TerminalCast cast={project.terminalCast} title="{project.name} - demo" />
  </div>
{:else if project.private}
  <div class={css({ mt: "20px" })}>
    <ClosedSourceCallout />
  </div>
{/if}
