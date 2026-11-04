<script lang="ts">
	import { api } from '#lib/api/client';
	import type { AlbumSummary } from '#lib/api/bindings/AlbumSummary';
	import type { LocalAlbumSummary } from '#lib/api/bindings/LocalAlbumSummary';
	import type { LocalAsset } from '#lib/api/bindings/LocalAsset';
	import { app } from '#lib/state/app.svelte';
	import AssetThumb from './AssetThumb.svelte';
	import Icon from './Icon.svelte';

	let {
		open = $bindable(false),
		initialDest = $bindable(''),
		initialSelection = [],
		onclose,
		onuploaded,
	}: {
		open?: boolean;
		initialDest?: string;
		initialSelection?: string[];
		onclose?: () => void;
		onuploaded?: () => void;
	} = $props();

	let assets = $state<LocalAsset[]>([]);
	let albums = $state<AlbumSummary[]>([]);
	let localAlbums = $state<LocalAlbumSummary[]>([]);
	let selected = $state<string[]>([]);
	let dest = $state(initialDest);
	let loading = $state(false);
	let uploading = $state(false);
	// `null` means the whole device, otherwise a registered local album's uri.
	let source = $state<string | null>(null);

	$effect(() => {
		if (!open) return;
		dest = initialDest;
		selected = [...initialSelection];
		void load();
	});

	async function load() {
		loading = true;
		try {
			const [local, remote, registered] = await Promise.all([
				api.localAssetPage({
					albumUri: source ?? '',
					cursor: null,
					limit: 200,
				}),
				api.albumList(),
				api.localAlbumList(),
			]);
			assets = local.items;
			albums = remote;
			localAlbums = registered;
			if (!dest && albums.length === 1) dest = albums[0]!.id;
		} catch (err) {
			app.showToast(app.describeError(err), 'error');
		} finally {
			loading = false;
		}
	}

	async function switchSource(uri: string | null) {
		source = uri;
		selected = [];
		await load();
	}

	function toggle(asset: LocalAsset) {
		selected = selected.includes(asset.id)
			? selected.filter((id) => id !== asset.id)
			: [...selected, asset.id];
	}

	function destName(): string {
		return albums.find((album) => album.id === dest)?.name ?? '选择相册';
	}

	async function upload() {
		if (!selected.length || !dest || uploading) return;
		uploading = true;
		try {
			const tasks = await api.enqueueUploads({
				localIds: selected,
				albumId: dest,
			});
			app.showToast(`已加入上传队列 · ${tasks.length} 项 → ${destName()}`, 'success');
			open = false;
			onuploaded?.();
			onclose?.();
		} catch (err) {
			app.showToast(app.describeError(err), 'error');
		} finally {
			uploading = false;
		}
	}

	function close() {
		open = false;
		onclose?.();
	}
</script>

{#if open}
	<div
		class="fixed inset-0 z-50 flex items-end justify-center bg-black/50"
		role="dialog"
		aria-modal="true"
	>
		<div class="flex h-[88dvh] w-full max-w-2xl flex-col rounded-t-3xl bg-light">
			<div class="flex items-center justify-between px-4 pb-2 pt-3">
				<div>
					<h2 class="text-base font-semibold">上传到相册</h2>
					<p class="text-xs text-muted">选择本机照片 · 原图上传</p>
				</div>
				<button
					onclick={close}
					class="grid size-9 place-items-center rounded-full bg-gray text-dark hover:bg-line/40"
					aria-label="关闭"
				>
					<Icon name="close" size={18} />
				</button>
			</div>

			{#if localAlbums.length > 0}
				<div class="flex gap-2 overflow-x-auto px-4 py-2">
					<button
						onclick={() => void switchSource(null)}
						class="shrink-0 rounded-lg px-3 py-1.5 text-xs font-medium transition-colors {source === null
							? 'bg-primary text-on-primary'
							: 'bg-gray text-muted hover:text-dark'}"
					>
						全部
					</button>
					{#each localAlbums as album (album.id)}
						<button
							onclick={() => void switchSource(album.uri)}
							class="shrink-0 rounded-lg px-3 py-1.5 text-xs font-medium transition-colors {source ===
							album.uri
								? 'bg-primary text-on-primary'
								: 'bg-gray text-muted hover:text-dark'}"
						>
							{album.name}
							{album.assetCount}
						</button>
					{/each}
				</div>
			{/if}

			<div class="min-h-0 flex-1 overflow-y-auto px-1">
				{#if loading}
					<div class="grid grid-cols-4 gap-[2px]">
						{#each Array(16) as _}
							<div class="aspect-square animate-pulse bg-gray"></div>
						{/each}
					</div>
				{:else if assets.length === 0}
					<div class="flex flex-col items-center gap-2 py-16 text-sm text-muted">
						<Icon name="image" size={30} />
						<p>没有可上传的本机照片</p>
						<p class="text-[11px] text-muted">先在资料库里同步一个本机相册</p>
					</div>
				{:else}
					<div class="grid grid-cols-4 gap-[2px] sm:grid-cols-5">
						{#each assets as asset (asset.id)}
							{@const video = asset.mime?.startsWith('video') ?? false}
							<AssetThumb
								source={{ kind: 'localThumb', uri: asset.uri }}
								name={asset.name}
								{video}
								selectionMode
								selected={selected.includes(asset.id)}
								ontoggle={() => toggle(asset)}
							/>
						{/each}
					</div>
				{/if}
			</div>

			<div class="border-t border-line px-4 pb-[calc(0.75rem+var(--safe-bottom))] pt-3">
				<label class="mb-2 flex items-center gap-2 text-xs text-muted">
					<span class="shrink-0">上传到</span>
					<select
						bind:value={dest}
						class="min-w-0 flex-1 rounded-lg border border-line bg-transparent px-3 py-1.5 text-xs outline-none focus:ring-2 focus:ring-primary/60"
					>
						<option value="">选择相册</option>
						{#each albums as album (album.id)}
							<option value={album.id}>{album.name}</option>
						{/each}
					</select>
				</label>
				<button
					onclick={() => void upload()}
					disabled={selected.length === 0 || !dest || uploading}
					class="flex h-11 w-full items-center justify-center gap-2 rounded-xl bg-primary text-sm font-semibold text-on-primary transition-opacity hover:opacity-90 disabled:opacity-40"
				>
					<Icon name="upload" size={18} />
					{uploading
						? '正在加入队列…'
						: dest
							? `上传 ${selected.length} 项到 ${destName()}`
							: '先选择一个相册'}
				</button>
			</div>
		</div>
	</div>
{/if}
