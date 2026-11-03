<script lang="ts">
	import { page } from '$app/state';
	import { api } from '#lib/api/client';
	import type { LocalAlbum } from '#lib/api/bindings/LocalAlbum';
	import type { LocalAsset } from '#lib/api/bindings/LocalAsset';
	import AssetThumb from '#lib/components/AssetThumb.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import TopBar from '#lib/components/TopBar.svelte';
	import UploadSheet from '#lib/components/UploadSheet.svelte';
	import { formatBytes } from '#lib/format';
	import { localThumb } from '#lib/api/media';
	import { app } from '#lib/state/app.svelte';

	const PAGE = 200;

	let album = $state<LocalAlbum | null>(null);
	let items = $state<LocalAsset[]>([]);
	let next = $state<string | null>(null);
	let loading = $state(true);
	let loadingMore = $state(false);
	let sentinel: HTMLDivElement | undefined = $state();
	let selected = $state<string[]>([]);
	let uploadOpen = $state(false);
	let initialDest = $state('');
	let currentId = $state('');

	$effect(() => {
		const id = page.params.id ?? '';
		if (id && id !== currentId) {
			currentId = id;
			void load(id);
		}
	});

	$effect(() => {
		if (!sentinel || !next) return;
		const observer = new IntersectionObserver(
			(entries) => {
				if (entries[0]?.isIntersecting) void loadMore();
			},
			{ rootMargin: '800px 0px' },
		);
		observer.observe(sentinel);
		return () => observer.disconnect();
	});

	async function load(id: string) {
		loading = true;
		next = null;
		try {
			const albums = await api.localAlbumList();
			album = albums.find((entry) => entry.id === id) ?? null;
			const result = await api.localAssetPage({ albumId: id, cursor: null, limit: PAGE });
			items = result.items;
			next = result.next;
		} catch (err) {
			app.showToast(app.describeError(err), 'error');
		} finally {
			loading = false;
		}
	}

	async function loadMore() {
		if (!next || loadingMore) return;
		loadingMore = true;
		const id = currentId;
		const cursor = next;
		try {
			const result = await api.localAssetPage({ albumId: id, cursor, limit: PAGE });
			if (id !== currentId) return;
			items = [...items, ...result.items];
			next = result.next;
		} catch (err) {
			if (id === currentId) app.showToast(app.describeError(err), 'error');
		} finally {
			loadingMore = false;
		}
	}

	function toggle(asset: LocalAsset) {
		selected = selected.includes(asset.id)
			? selected.filter((id) => id !== asset.id)
			: [...selected, asset.id];
	}

	function openUpload() {
		initialDest = '';
		uploadOpen = true;
	}

	function uploaded() {
		selected = [];
		app.showToast('上传任务已创建，可在资料库查看进度', 'success');
	}
</script>

<svelte:head><title>{album?.name ?? '本机相册'} · Source</title></svelte:head>

<TopBar
	title={selected.length ? `已选择 ${selected.length} 项` : (album?.name ?? '本机相册')}
	subtitle={loading ? '加载中…' : `${album?.assetCount ?? items.length} 项 · 本机`}
	back={selected.length === 0}
>
	{#snippet actions()}
		{#if selected.length > 0}
			<button
				onclick={() => {
					selected = [];
				}}
				class="grid size-9 place-items-center rounded-full text-dark hover:bg-gray"
				aria-label="取消选择"
			>
				<Icon name="close" size={21} />
			</button>
		{:else}
			<button
				onclick={openUpload}
				class="grid size-9 place-items-center rounded-full text-dark hover:bg-gray"
				aria-label="上传"
			>
				<Icon name="upload" size={21} />
			</button>
		{/if}
	{/snippet}
</TopBar>

{#if loading}
	<div class="grid grid-cols-3 gap-[2px]">
		{#each Array(12) as _}
			<div class="aspect-square animate-pulse bg-gray"></div>
		{/each}
	</div>
{:else if items.length === 0}
	<div class="flex flex-col items-center gap-3 px-8 py-20 text-center">
		<Icon name="folder" size={34} class="text-muted" />
		<p class="text-sm font-medium">这个相册是空的</p>
		<button
			onclick={openUpload}
			class="rounded-xl bg-primary px-4 py-2 text-xs font-medium text-on-primary"
			>选择要上传的照片</button
		>
	</div>
{:else}
	<div class="grid grid-cols-3 gap-[2px] sidebar:grid-cols-5">
		{#each items as asset (asset.id)}
			{@const video = asset.mime?.startsWith('video') ?? false}
			<AssetThumb
				src={localThumb(asset)}
				name={asset.name}
				{video}
				badge={formatBytes(asset.size)}
				selectionMode
				selected={selected.includes(asset.id)}
				ontoggle={() => toggle(asset)}
			/>
		{/each}
	</div>

	{#if next}
		<div bind:this={sentinel} class="flex justify-center pb-24 pt-2">
			<div class="size-6 animate-spin rounded-full border-2 border-line border-t-primary"></div>
		</div>
	{/if}
{/if}

{#if selected.length > 0}
	<div
		class="fixed inset-x-0 bottom-[calc(3.6rem+var(--safe-bottom))] z-20 border-t border-line bg-light/95 px-4 py-2 backdrop-blur sidebar:bottom-0 sidebar:ps-60"
	>
		<div class="mx-auto flex max-w-3xl items-center justify-between">
			<span class="text-xs text-muted">已选 {selected.length} 项</span>
			<button
				onclick={openUpload}
				class="flex items-center gap-2 rounded-xl bg-primary px-4 py-2 text-sm font-medium text-on-primary"
			>
				<Icon name="upload" size={17} /> 上传到相册
			</button>
		</div>
	</div>
{/if}

<UploadSheet
	bind:open={uploadOpen}
	bind:initialDest
	initialSelection={selected}
	onuploaded={uploaded}
/>
