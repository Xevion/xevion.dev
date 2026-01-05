<script lang="ts">
	import type { OGImageSpec } from '$lib/og-types';

	type Props = {
		title: string;
		subtitle?: string;
		type: OGImageSpec['type'];
	};

	let { title, subtitle, type }: Props = $props();

	// Calculate font size based on title length (matching original logic)
	const fontSize = $derived(title.length > 40 ? '60px' : '72px');
</script>

<div
	style="display: flex; width: 1200px; height: 630px; background-color: #000000; color: #fafafa; font-family: 'Schibsted Grotesk', sans-serif; padding: 60px 80px;"
>
	<div
		style="display: flex; flex-direction: column; justify-content: space-between; width: 100%; height: 100%;"
	>
		<!-- Content section -->
		<div style="display: flex; flex-direction: column; flex: 1; justify-content: center;">
			<h1
				style="font-family: 'Hanken Grotesk', sans-serif; font-weight: 900; font-size: {fontSize}; line-height: 1.1; margin: 0; color: #ffffff;"
			>
				{title}
			</h1>
			{#if subtitle}
				<p
					style="font-family: 'Schibsted Grotesk', sans-serif; font-size: 36px; margin: 32px 0 0 0; color: #a1a1aa; line-height: 1.4;"
				>
					{subtitle}
				</p>
			{/if}
		</div>

		<!-- Footer -->
		<div
			style="display: flex; justify-content: space-between; align-items: flex-end; border-top: 2px solid #27272a; padding-top: 24px;"
		>
			<div style="font-size: 28px; color: #71717a; font-weight: 500;">xevion.dev</div>
			{#if type === 'project'}
				<div
					style="font-size: 24px; color: #52525b; text-transform: uppercase; letter-spacing: 0.05em;"
				>
					PROJECT
				</div>
			{/if}
		</div>
	</div>
</div>
