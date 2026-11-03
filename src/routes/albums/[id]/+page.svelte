<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { api } from '#lib/api/client';
	import type { AlbumSummary } from '#lib/api/bindings/AlbumSummary';
	import type { AssetSummary } from '#lib/api/bindings/AssetSummary';
	import AlbumMenu from '#lib/components/AlbumMenu.svelte';
	import AlbumPickerSheet from '#lib/components/AlbumPickerSheet.svelte';
	import AssetGrid from '#lib/components/AssetGrid.svelte';
	import DescriptionDialog from '#lib/components/DescriptionDialog.svelte';
	import EmptyState from '#lib/components/EmptyState.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import SelectionBar from '#lib/components/SelectionBar.svelte';
	import TopBar from '#lib/components/TopBar.svelte';
	import Viewer from '#lib/components/Viewer.svelte';
	import { app } from '#lib/state/app.svelte';
	import { changes } from '#lib/state/changes';
	import { Selection } from '#lib/state/selection.svelte';
	import { Timeline } from '#lib/state/timeline.svelte';

	let currentId = $state('');
	const timeline = new Timeline((cursor, limit) =>
		api.albumPage({ albumId: currentId, cursor, limit }),
	);
	const selection = new Selection();

	let album = $state<AlbumSummary | null>(null);
	let metaError = $state('');
	let menuOpen = $state(false);
	let coverMode = $state(false);
	let viewerOpen = $state(false);
	let viewerIndex = $state(0);
	let descriptionOpen = $state(false);
	let descriptionDraft = $state('');
	let saving = $state(false);
	let pickerOpen = $state(false);
	let moving = $state(false);

	$effect(() => {
		const id = page.params.id;
		if (id && id !== currentId) {
			currentId = id;
			selection.exit();
			void load(id);
		}
	});

	$effect(() => {
		return changes.subscribe((change) => {
			if (change.kind === 'folders') {
				void load(currentId, true);
				return;
			}
			if (change.kind === 'albums') {
				const updated = change.albums.find((entry) => entry.id === currentId);
				if (updated) album = updated;
				return;
			}
			for (const asset of change.assets) {
				if (asset.folderId === currentId) timeline.upsert(asset);
			}
		});
	});

	async function load(id: string, keepItems = false) {
		if (!keepItems) timeline.reset();
		metaError = '';
		try {
			const albums = await api.albumList();
			album = albums.find((entry) => entry.id === id) ?? null;
			if (!album) {
				metaError = '相册不存在或已被移动';
				return;
			}
			descriptionDraft = album.description ?? '';
			await timeline.reload();
		} catch (err) {
			metaError = app.describeError(err);
		}
	}

	function open(asset: AssetSummary) {
		if (coverMode) {
			void setCover(asset);
			return;
		}
		if (selection.active) {
			selection.toggle(asset.id);
			return;
		}
		viewerIndex = timeline.items.findIndex((item) => item.id === asset.id);
		if (viewerIndex >= 0) viewerOpen = true;
	}

	async function setCover(asset: AssetSummary) {
		if (!album) return;
		try {
			album = await api.setAlbumCover({ albumId: album.id, assetId: asset.id });
			coverMode = false;
			app.showToast('封面已保存到 Cloudreve metadata', 'success');
		} catch (err) {
			app.showToast(app.describeError(err), 'error');
		}
	}

	async function saveDescription() {
		if (!album) return;
		saving = true;
		try {
			album = await api.setAlbumDescription({
				albumId: album.id,
				description: descriptionDraft.trim(),
			});
			descriptionOpen = false;
			app.showToast('描述已保存', 'success');
		} catch (err) {
			app.showToast(app.describeError(err), 'error');
		} finally {
			saving = false;
		}
	}

	async function downloadSelected() {
		if (!selection.count) return;
		try {
			const result = await api.exportAssets({ assetIds: selection.ids });
			app.showToast(`已导出 ${result.exported} 项`, 'success');
			selection.exit();
		} catch (err) {
			app.showToast(app.describeError(err), 'error');
		}
	}

	async function moveToAlbum(target: AlbumSummary) {
		moving = true;
		const ids = [...selection.ids];
		try {
			const moved = await api.moveAssetsToAlbum({
				assetIds: ids,
				albumId: target.id,
			});
			// The assets left this album's folder, so drop them from the grid.
			timeline.remove(ids);
			app.showToast(`已移动 ${moved.length} 项到「${target.name}」`, 'success');
			selection.exit();
		} catch (err) {
			app.showToast(app.describeError(err), 'error');
		} finally {
			moving = false;
		}
	}
</script>

<svelte:head><title>{album ? `${album.name} · 相册` : '相册'} · Source</title></svelte:head>

<TopBar
	title={coverMode
		? '选择封面'
		: selection.active
			? `已选择 ${selection.count} 项`
			: (album?.name ?? '相册')}
	subtitle={coverMode
		? '点击任意照片设为相册封面'
		: selection.active
			? ''
			: album
				? `${album.assetCount} 项 · ${album.path}`
				: ''}
	back={!coverMode && !selection.active}
>
	{#snippet actions()}
		{#if coverMode}
			<button
				onclick={() => (coverMode = false)}
				class="rounded-lg px-3 py-2 text-xs font-medium text-muted hover:bg-gray"
				>取消</button
			>
		{:else if selection.active}
			<button
				onclick={() => selection.toggleAll(timeline.items.map((item) => item.id))}
				class="rounded-lg px-3 py-2 text-xs font-medium text-primary"
			>
				{selection.count === timeline.items.length ? '取消全选' : '全选'}
			</button>
			<button
				onclick={() => selection.exit()}
				class="grid size-9 place-items-center rounded-full text-dark hover:bg-gray"
				aria-label="取消"
			>
				<Icon name="close" size={21} />
			</button>
		{:else}
			<button
				onclick={() => selection.start()}
				class="grid size-9 place-items-center rounded-full text-dark hover:bg-gray"
				aria-label="选择"
			>
				<Icon name="select" size={20} />
			</button>
			<button
				onclick={() => (menuOpen = true)}
				class="grid size-9 place-items-center rounded-full text-dark hover:bg-gray"
				aria-label="相册操作"
			>
				<Icon name="more" size={22} />
			</button>
		{/if}
	{/snippet}
</TopBar>

{#if album?.description && !coverMode}
	<p class="whitespace-pre-wrap px-4 pt-2 text-sm text-muted">
		{album.description}
	</p>
{/if}

{#if timeline.loading && timeline.items.length === 0 && !timeline.loaded}
	<div class="grid grid-cols-3 gap-[2px]">
		{#each Array(12) as _}
			<div class="aspect-square animate-pulse bg-gray"></div>
		{/each}
	</div>
{:else if metaError || timeline.error}
	<EmptyState
		icon="wifi-off"
		title="无法加载相册"
		description={metaError || timeline.error}
		actionLabel="重试"
		onaction={() => void load(currentId)}
	/>
{:else if timeline.items.length === 0}
	<EmptyState
		icon="image"
		title="这个相册还是空的"
		description="从本机上传照片，或把时间线里的照片移动进来。"
		actionLabel="上传照片"
		onaction={() => void goto('/library?upload=1')}
	/>
{:else}
	<AssetGrid
		items={timeline.items}
		columns={app.columns}
		selectionMode={selection.active}
		selected={selection.ids}
		onopen={open}
		ontoggle={(asset) => selection.toggle(asset.id)}
		oncolumns={(columns) => app.setColumns(columns)}
	/>
	{#if timeline.next}
		<button
			onclick={() => void timeline.loadMore()}
			class="mx-auto mb-28 mt-2 block rounded-xl border border-line px-4 py-2 text-xs text-muted transition-colors hover:bg-gray"
		>
			加载更多
		</button>
	{/if}
{/if}

{#if selection.active}
	<SelectionBar
		count={selection.count}
		busy={moving}
		onmove={() => (pickerOpen = true)}
		ondownload={() => void downloadSelected()}
		oncancel={() => selection.exit()}
	/>
{/if}

<AlbumPickerSheet
	bind:open={pickerOpen}
	excludeId={currentId}
	onpick={(target) => void moveToAlbum(target)}
/>

<AlbumMenu
	bind:open={menuOpen}
	oncover={() => (coverMode = true)}
	ondescription={() => (descriptionOpen = true)}
	onselect={() => selection.start()}
/>

<DescriptionDialog
	bind:open={descriptionOpen}
	bind:draft={descriptionDraft}
	{saving}
	onsave={() => void saveDescription()}
/>

{#if viewerOpen}
	<Viewer items={timeline.items} bind:index={viewerIndex} onclose={() => (viewerOpen = false)} />
{/if}
