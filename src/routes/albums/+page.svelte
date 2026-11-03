<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { api } from '#lib/api/client';
	import type { AlbumSummary } from '#lib/api/bindings/AlbumSummary';
	import AlbumCard from '#lib/components/AlbumCard.svelte';
	import EmptyState from '#lib/components/EmptyState.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import IndexButton from '#lib/components/IndexButton.svelte';
	import TopBar from '#lib/components/TopBar.svelte';
	import { app } from '#lib/state/app.svelte';
	import { changes } from '#lib/state/changes';

	let albums = $state<AlbumSummary[]>([]);
	let loading = $state(true);
	let error = $state('');

	$effect(() => {
		// Counts and mosaics come from assets too, so any catalog write refreshes
		// this grid; keyed each-blocks keep the scroll position intact.
		return changes.subscribe(() => void load());
	});

	onMount(() => {
		void load();
	});

	async function load() {
		loading = true;
		error = '';
		try {
			albums = await api.albumList();
		} catch (err) {
			error = app.describeError(err);
		} finally {
			loading = false;
		}
	}
</script>

<svelte:head><title>相册 · Source</title></svelte:head>

<TopBar title="相册" subtitle={loading ? '正在加载…' : `${albums.length} 个目录`}>
	{#snippet actions()}
		<button
			onclick={() => void goto('/library?upload=1')}
			class="grid size-9 place-items-center rounded-full text-dark transition-colors hover:bg-gray"
			aria-label="上传"
		>
			<Icon name="upload" size={21} />
		</button>
		<IndexButton />
	{/snippet}
</TopBar>

{#if !app.account?.libraryRoot}
	<EmptyState
		icon="albums"
		title="先选择图库根目录"
		description="图库根目录下的每个文件夹都会成为一个相册。"
		actionLabel="去资料库设置"
		onaction={() => void goto('/library?setup=1')}
	/>
{:else if loading && albums.length === 0}
	<div class="grid grid-cols-2 gap-3 p-4">
		{#each Array(4) as _}
			<div class="aspect-square animate-pulse rounded-2xl bg-gray"></div>
		{/each}
	</div>
{:else if error}
	<EmptyState
		icon="wifi-off"
		title="无法加载相册"
		description={error}
		actionLabel="重试"
		onaction={() => void load()}
	/>
{:else if albums.length === 0}
	<EmptyState
		icon="albums"
		title="还没有相册"
		description="运行索引后，Library Root 下的文件夹会出现在这里。"
		actionLabel="开始索引"
		onaction={() => void app.startIndex(false)}
	/>
{:else}
	<div class="px-4 pb-24 pt-2">
		<p class="mb-2 px-1 text-xs text-muted">
			目录即相册 · 封面与描述保存在 Cloudreve metadata
		</p>
		<div class="grid grid-cols-2 gap-3 sm:grid-cols-3 sidebar:grid-cols-4">
			{#each albums as album (album.id)}
				<AlbumCard {album} />
			{/each}
		</div>
	</div>
{/if}
