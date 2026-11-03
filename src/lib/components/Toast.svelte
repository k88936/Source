<script lang="ts">
	import { app } from '#lib/state/app.svelte';
	import Icon from './Icon.svelte';

	const tone = $derived(
		app.toast?.kind === 'error'
			? 'bg-red-600'
			: app.toast?.kind === 'success'
				? 'bg-emerald-600'
				: 'bg-dark',
	);
</script>

{#if app.toast}
	<div class="pointer-events-none fixed inset-x-0 bottom-24 z-[60] flex justify-center px-4">
		<div
 		class="flex max-w-sm items-center gap-2 rounded-xl px-4 py-2.5 text-sm text-white shadow-lg {tone}"
		>
			<Icon name={app.toast.kind === 'error' ? 'info' : 'check-circle'} size={17} />
			<span>{app.toast.message}</span>
		</div>
	</div>
{/if}
