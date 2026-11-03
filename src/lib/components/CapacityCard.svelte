<script lang="ts">
	import Icon from './Icon.svelte';
	import { formatBytes } from '#lib/format';
	import { app } from '#lib/state/app.svelte';

	const usedPercent = $derived.by(() => {
		const capacity = app.session?.capacity;
		if (!capacity || capacity.total <= 0) return 0;
		return Math.min(100, Math.round((capacity.used / capacity.total) * 100));
	});
</script>

<a href="/settings" class="block rounded-2xl bg-gray p-4 transition-colors hover:bg-line/40">
	<div class="flex items-center gap-3">
		<div
			class="grid size-12 shrink-0 place-items-center rounded-full bg-primary text-lg font-semibold text-on-primary"
		>
			{app.session?.user.name?.slice(0, 1).toUpperCase() ?? '?'}
		</div>
		<div class="min-w-0 flex-1">
			<p class="truncate text-base font-semibold">{app.session?.user.name}</p>
			<p class="truncate text-xs text-muted">
				{app.session?.siteName ?? app.session?.server}
			</p>
		</div>
		<Icon name="settings" size={19} class="text-muted" />
	</div>

	<div class="mt-4">
		<div class="mb-1 flex justify-between text-[11px] text-muted">
			<span>存储空间</span>
			<span>
				{#if app.session?.capacity}
					{formatBytes(app.session.capacity.used)} / {formatBytes(app.session.capacity.total)}
				{:else}
					未知
				{/if}
			</span>
		</div>
		<div class="h-1.5 overflow-hidden rounded-full bg-muted/25">
			<div class="h-full rounded-full bg-primary" style:width="{usedPercent}%"></div>
		</div>
	</div>
</a>
