<script lang="ts">
	import Icon from './Icon.svelte';

	let {
		count,
		busy = false,
		onmove,
		ondownload,
		oncancel,
	}: {
		count: number;
		busy?: boolean;
		onmove?: () => void;
		ondownload: () => void;
		oncancel: () => void;
	} = $props();
</script>

<div
	class="fixed inset-x-0 bottom-[calc(3.6rem+var(--safe-bottom))] z-20 border-t border-line bg-light/95 px-4 py-2 backdrop-blur sidebar:bottom-0 sidebar:ps-60"
>
	<div class="mx-auto flex max-w-3xl items-center justify-between gap-2">
		<button onclick={oncancel} class="rounded-xl px-3 py-2 text-sm text-muted hover:bg-gray">
			取消
		</button>
		<div class="flex items-center gap-2">
			{#if onmove}
				<button
					onclick={onmove}
					disabled={busy || count === 0}
					class="flex items-center gap-1.5 rounded-xl border border-line px-4 py-2 text-sm font-medium text-dark transition-colors hover:bg-gray disabled:opacity-40"
				>
					<Icon name="albums" size={16} /> 移动到相册
				</button>
			{/if}
			<button
				onclick={ondownload}
				disabled={busy || count === 0}
				class="flex items-center gap-2 rounded-xl bg-primary px-4 py-2 text-sm font-medium text-on-primary transition-opacity hover:opacity-90 disabled:opacity-40"
			>
				<Icon name="download" size={17} /> 下载
			</button>
		</div>
	</div>
</div>
