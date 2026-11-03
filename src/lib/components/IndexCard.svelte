<script lang="ts">
	import Icon from './Icon.svelte';
	import { app } from '#lib/state/app.svelte';

	const percent = $derived(
		app.index.foldersTotal
			? Math.round((app.index.foldersDone / app.index.foldersTotal) * 100)
			: app.index.assetsSeen > 0
				? 100
				: 0,
	);
	const hint = $derived(
		app.index.phase === 'failed'
			? (app.index.error?.message ?? '索引失败')
			: app.scanning
				? '可继续浏览，索引在后台进行'
				: app.index.phase === 'paused'
					? '已暂停，继续时从没扫完的目录接着走'
					: '增量扫描会跳过未变化的目录',
	);
</script>

<section class="mt-3 rounded-2xl bg-gray p-4">
	<div class="flex items-start gap-3">
		<div
			class="grid size-10 shrink-0 place-items-center rounded-xl bg-primary/10 text-primary"
		>
			<Icon name="sync" size={21} class={app.scanning ? 'animate-spin' : ''} />
		</div>
		<div class="min-w-0 flex-1">
			<div class="flex items-center justify-between gap-2">
				<p class="text-sm font-semibold">
					{app.scanning
						? `正在索引 ${app.index.foldersDone}/${app.index.foldersTotal}`
						: '图库索引'}
				</p>
				<span class="text-[11px] text-muted">{app.index.assetsSeen} 项</span>
			</div>
			<div class="mt-2 h-1.5 overflow-hidden rounded-full bg-muted/25">
				<div
					class="h-full rounded-full bg-primary transition-all"
					style:width="{percent}%"
				></div>
			</div>
			<p class="mt-1.5 text-[11px] text-muted">{hint}</p>
		</div>
	</div>
	<div class="mt-3 flex gap-2">
		{#if app.scanning}
			<button
				onclick={() => void app.cancelIndex()}
				class="flex flex-1 items-center justify-center gap-1.5 rounded-xl border border-line py-2 text-xs font-semibold hover:bg-gray"
			>
				<Icon name="stop" size={13} /> 暂停
			</button>
		{:else}
			<button
				onclick={() => void app.startIndex(false)}
				disabled={!app.account?.libraryRoot}
				class="flex-1 rounded-xl bg-primary py-2 text-xs font-semibold text-on-primary transition-opacity hover:opacity-90 disabled:opacity-40"
			>
				{app.index.phase === 'paused' ? '继续索引' : '增量索引'}
			</button>
			<button
				onclick={() => void app.startIndex(true)}
				disabled={!app.account?.libraryRoot}
				class="rounded-xl border border-line px-3 py-2 text-xs transition-colors hover:bg-gray"
			>
				完整重建
			</button>
		{/if}
	</div>
</section>
