<script lang="ts" module>
	import { renderIconSVG } from "$lib/server/icons";
</script>

<script lang="ts">
	import { cn } from "$lib/utils";

	interface Props {
		icon: string;
		class?: string;
		size?: number;
		fallback?: string;
	}

	let { icon, class: className, size, fallback = "lucide:help-circle" }: Props = $props();
</script>

{#await renderIconSVG(icon, { class: cn("inline-block", className), size })}
	<!-- Loading state during SSR (shouldn't be visible) -->
{:then svg}
	{#if svg}
		{@html svg}
	{:else}
		<!-- Fallback icon if primary fails -->
		{#await renderIconSVG(fallback, { class: cn("inline-block", className), size }) then fallbackSvg}
			{@html fallbackSvg}
		{/await}
	{/if}
{/await}
