<script lang="ts">
	import type { IndexStatus } from '#lib/api/bindings/IndexStatus';
	import { app } from '#lib/state/app.svelte';
	import Icon from './Icon.svelte';

	let open = $state(false);
	let busy = $state(false);
	let full = $state(false);

	let status = $derived<IndexStatus>(app.index);
	let percent = $derived(
		status.foldersTotal > 0
			? Math.min(100, Math.round((status.foldersDone / status.foldersTotal) * 100))
			: 0,
	);

	async function start() {
		busy = true;
		await app.startIndex(full);
		busy = false;
		open = false;
	}

	function statusText(): string {
		switch (status.phase) {
			case 'scanning':
				return `正在索引 ${status.foldersDone}/${status.foldersTotal}`;
			case 'paused':
				return `已暂停 · ${status.foldersDone}/${status.foldersTotal}`;
			case 'failed':
				return status.error?.message ?? '索引失败';
			default:
				return status.assetsSeen > 0 ? `已索引 ${status.assetsSeen} 项` : '尚未索引';
		}
	}
</script>

<button
	onclick={() => (open = true)}
	class="relative grid size-9 place-items-center rounded-full text-dark transition-colors hover:bg-gray"
	aria-label="索引状态"
	title={statusText()}
>
	<Icon name="sync" size={21} class={status.phase === 'scanning' ? 'animate-spin' : ''} />
	{#if status.phase === 'failed'}
		<span class="absolute right-1.5 top-1.5 size-2 rounded-full bg-red-500"></span>
	{:else if status.phase === 'scanning'}
		<span class="absolute right-1.5 top-1.5 size-2 animate-pulse rounded-full bg-primary"></span>
	{/if}
</button>

{#if open}
	<div
		class="fixed inset-0 z-50 flex items-end justify-center bg-black/45 p-0 sm:items-center sm:p-6"
		role="dialog"
		aria-modal="true"
		tabindex="-1"
		onclick={(e) => {
			if (e.target === e.currentTarget) open = false;
		}}
		onkeydown={(e) => {
			if (e.key === 'Escape') open = false;
		}}
	>
		<div
 		class="w-full max-w-md rounded-t-3xl bg-light p-5 shadow-2xl sm:rounded-3xl"
		>
			<div class="mb-4 flex items-center justify-between">
  		<h2 class="text-lg font-semibold">图库索引</h2>
				<button
					class="rounded-full p-2 hover:bg-gray"
					onclick={() => (open = false)}
					aria-label="关闭"
				>
					<Icon name="close" size={20} />
				</button>
			</div>

			<div class="rounded-2xl bg-gray p-4">
				<div class="mb-2 flex items-baseline justify-between">
					<span class="text-sm font-medium">{statusText()}</span>
					{#if status.phase === 'scanning'}
 					<span class="text-xs text-muted">{percent}%</span>
					{/if}
				</div>
				<div class="h-1.5 overflow-hidden rounded-full bg-muted/25">
					<div
						class="h-full rounded-full bg-primary transition-all"
						style:width="{status.phase === 'scanning'
							? percent
							: status.phase === 'failed'
								? 100
								: 100}%"
					></div>
				</div>
				<p class="mt-2 text-xs text-muted">
					目录 {status.foldersDone}/{status.foldersTotal} · 资产 {status.assetsSeen}
				</p>
				{#if status.error}
					<p class="mt-2 text-xs text-red-500">{status.error.message}</p>
				{/if}
			</div>

			<label class="mt-4 flex items-center gap-3 rounded-xl px-1 py-2 text-sm">
				<input type="checkbox" bind:checked={full} class="size-4 accent-primary" />
				<span>完整重建（忽略增量剪枝）</span>
			</label>

			<div class="mt-3 flex gap-2">
				{#if status.phase === 'scanning'}
					<button
						onclick={() => void app.cancelIndex()}
						class="flex flex-1 items-center justify-center gap-2 rounded-xl border border-line px-4 py-2.5 text-sm font-semibold hover:bg-gray"
					>
						<Icon name="stop" size={15} /> 暂停索引
					</button>
				{:else}
					<button
						onclick={start}
						disabled={busy}
						class="flex-1 rounded-xl bg-primary px-4 py-2.5 text-sm font-semibold text-on-primary transition-opacity hover:opacity-90 disabled:opacity-50"
					>
						{busy ? '启动中…' : status.phase === 'paused' ? '继续索引' : '开始索引'}
					</button>
				{/if}
				<button
					onclick={() => void app.refreshIndex()}
 				class="rounded-xl border border-line px-4 py-2.5 text-sm transition-colors hover:bg-gray"
				>
					刷新
				</button>
			</div>
			<p class="mt-3 text-[11px] leading-relaxed text-muted">
				索引在后台渐进执行，可继续浏览；暂停后会记住进度，继续时从没扫完的目录接着走。
			</p>
		</div>
	</div>
{/if}
