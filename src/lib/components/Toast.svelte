<script lang="ts">
	import { fly } from 'svelte/transition';
	import { app } from '#lib/state/app.svelte';
	import Icon from './Icon.svelte';

	const kind = $derived(app.toast?.kind ?? 'info');

	const accent = $derived(
		kind === 'error'
			? 'bg-danger/10 text-danger'
			: kind === 'success'
				? 'bg-success/10 text-success'
				: 'bg-info/10 text-info',
	);
</script>

{#if app.toast}
	<div class="pointer-events-none fixed inset-x-0 bottom-24 z-[60] flex justify-center px-4">
		<div
			transition:fly={{ y: 10, duration: 160 }}
			class="flex max-w-sm items-center gap-3 rounded-2xl bg-gray py-2.5 pe-4 ps-2.5 text-sm text-dark shadow-lg ring-1 ring-line"
		>
			<span class="grid size-7 shrink-0 place-items-center rounded-full {accent}">
				<Icon
					name={kind === 'success' ? 'check-circle' : kind === 'error' ? 'alert' : 'info'}
					size={16}
				/>
			</span>
			<span class="leading-snug">{app.toast.message}</span>
		</div>
	</div>
{/if}
