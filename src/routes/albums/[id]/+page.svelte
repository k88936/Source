<script lang="ts">
    import {goto} from '$app/navigation';
    import {page} from '$app/state';
    import {api} from '#lib/api/client';
    import type {AlbumSummary} from '#lib/api/bindings/AlbumSummary';
    import type {AssetSummary} from '#lib/api/bindings/AssetSummary';
    import AlbumMenu from '#lib/components/AlbumMenu.svelte';
    import AlbumPickerSheet from '#lib/components/AlbumPickerSheet.svelte';
    import AssetGrid from '#lib/components/AssetGrid.svelte';
    import ConfirmDialog from '#lib/components/ConfirmDialog.svelte';
    import DescriptionDialog from '#lib/components/DescriptionDialog.svelte';
    import EmptyState from '#lib/components/EmptyState.svelte';
    import {Ellipsis, Image, RefreshCw, SquareDashed, WifiOff, X} from '@lucide/svelte';
    import SelectAllButton from '#lib/components/SelectAllButton.svelte';
    import SelectionBar from '#lib/components/SelectionBar.svelte';
    import TopBar from '#lib/components/TopBar.svelte';
    import Viewer from '#lib/components/Viewer.svelte';
    import {app} from '#lib/state/app.svelte';
    import {changes} from '#lib/state/changes';
    import {Selection} from '#lib/state/selection.svelte';
    import {Timeline} from '#lib/state/timeline.svelte';

    let currentId = $state('');
    const timeline = new Timeline((cursor, limit) => api.albumPage({albumId: currentId, cursor, limit}));
    const selection = new Selection();

    let album = $state<AlbumSummary | null>(null);
    let metaError = $state('');
    let menuOpen = $state(false);
    let removeOpen = $state(false);
    let coverMode = $state(false);
    let viewerOpen = $state(false);
    let viewerIndex = $state(0);
    let descriptionOpen = $state(false);
    let descriptionDraft = $state('');
    let saving = $state(false);
    let pickerOpen = $state(false);
    let moving = $state(false);
    let refreshing = $state(false);

    const title = $derived.by(() => {
        if (coverMode) return '选择封面';
        if (selection.active) return `已选择 ${selection.count} 项`;
        return album?.name ?? '相册';
    });

    const subtitle = $derived.by(() => {
        if (coverMode) return '点击任意照片设为相册封面';
        if (selection.active) return '';
        if (!album) return '';
        return `${album.assetCount} 项 - ${album.path}`;
    });

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
            album = await api.albumGet({albumId: id});
            if (!album) {
                metaError = '相册不存在或已被移除';
                return;
            }
            descriptionDraft = album.description ?? '';
            await timeline.reload();
        } catch (err) {
            metaError = app.describeError(err);
        }
    }

    /// List this folder once. A failure means the folder is gone or renamed on
    /// the server: the album stays, its assets stay, and `sync.failed` shows the
    /// banner offering a retry.
    async function refresh() {
        if (refreshing) return;
        refreshing = true;
        try {
            album = await api.refreshAlbum({albumId: currentId});
            await timeline.reload();
            app.showToast('相册已刷新', 'success');
        } catch (err) {
            await load(currentId, true);
            app.showToast(app.describeError(err), 'error');
        } finally {
            refreshing = false;
        }
    }

    async function remove() {
        if (!album) return;
        removeOpen = false;
        try {
            await api.removeAlbum({albumId: album.id});
            app.showToast('相册已移除', 'success');
            await goto('/albums');
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
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
            album = await api.setAlbumCover({albumId: album.id, assetId: asset.id});
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
            const result = await api.exportAssets({assetIds: selection.ids});
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
            app.showToast(`已移动 ${moved.length} 项到 "${target.name}"`, 'success');
            selection.exit();
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        } finally {
            moving = false;
        }
    }
</script>

<svelte:head><title>{album ? `${album.name} - 相册` : '相册'} - Source</title></svelte:head>

<TopBar {title} {subtitle} back={!coverMode && !selection.active}>
    {#snippet actions()}
        {#if coverMode}
            <button
                onclick={() => (coverMode = false)}
                class="rounded-lg px-3 py-2 text-xs font-medium text-muted hover:bg-gray">取消</button
            >
        {:else if selection.active}
            <SelectAllButton {selection} ids={timeline.items.map((item) => item.id)} />
            <button
                onclick={() => selection.exit()}
                class="grid size-9 place-items-center rounded-full text-dark hover:bg-gray"
                aria-label="取消"
            >
                <X size={21} />
            </button>
        {:else}
            <button
                onclick={() => void refresh()}
                class="grid size-9 place-items-center rounded-full text-dark hover:bg-gray"
                aria-label="刷新相册"
            >
                {#if refreshing}
                    <div class="size-4 animate-spin rounded-full border-2 border-line border-t-primary"></div>
                {:else}
                    <RefreshCw size={20} />
                {/if}
            </button>
            <button
                onclick={() => selection.start()}
                class="grid size-9 place-items-center rounded-full text-dark hover:bg-gray"
                aria-label="选择"
            >
                <SquareDashed size={20} />
            </button>
            <button
                onclick={() => (menuOpen = true)}
                class="grid size-9 place-items-center rounded-full text-dark hover:bg-gray"
                aria-label="相册操作"
            >
                <Ellipsis size={22} />
            </button>
        {/if}
    {/snippet}
</TopBar>

{#if album?.sync?.failed && !coverMode}
    <div class="mx-4 mt-3 rounded-2xl border border-amber-500/40 bg-amber-500/10 px-3 py-2.5">
        <p class="text-xs font-medium text-amber-600">这个文件夹在服务器上已不可访问</p>
        <p class="mt-0.5 text-[11px] text-muted">
            {album.sync?.error ?? '目录可能已被删除或改名'} - 已保留上次索引的照片
        </p>
        <div class="mt-2 flex gap-2">
            <button
                onclick={() => void refresh()}
                class="rounded-lg bg-primary px-3 py-1.5 text-xs font-medium text-on-primary disabled:opacity-40"
                disabled={refreshing}>重试</button
            >
            <button
                onclick={() => (removeOpen = true)}
                class="rounded-lg border border-line px-3 py-1.5 text-xs text-muted hover:bg-gray">移除相册</button
            >
        </div>
    </div>
{/if}

{#if album?.description && !coverMode}
    <p class="whitespace-pre-wrap px-4 pt-2 text-sm text-muted">
        {album.description}
    </p>
{/if}

{#if timeline.loading && timeline.items.length === 0 && !timeline.loaded}
    <div class="grid grid-cols-3 gap-0.5">
        {#each Array(12) as _}
            <div class="aspect-square animate-pulse bg-gray"></div>
        {/each}
    </div>
{:else if metaError || timeline.error}
    <EmptyState
        icon={WifiOff}
        title="无法加载相册"
        description={metaError || timeline.error}
        actionLabel="重试"
        onAction={() => void load(currentId)}
    />
{:else if timeline.items.length === 0}
    <EmptyState
        icon={Image}
        title="这个相册还是空的"
        description="从本机上传照片, 或把时间线里的照片移动进来."
        actionLabel="上传照片"
        onAction={() => void goto('/library?upload=1')}
    />
{:else}
    <AssetGrid
        items={timeline.items}
        columns={app.columns}
        selectionMode={selection.active}
        selected={selection.ids}
        onOpen={open}
        onToggle={(asset) => selection.toggle(asset.id)}
        onColumns={(columns) => app.setColumns(columns)}
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
        isBusy={moving}
        onMove={() => (pickerOpen = true)}
        onDownload={() => void downloadSelected()}
        onCancel={() => selection.exit()}
    />
{/if}

<AlbumPickerSheet bind:open={pickerOpen} excludeId={currentId} onPick={(target) => void moveToAlbum(target)} />

<AlbumMenu
    bind:open={menuOpen}
    onCover={() => (coverMode = true)}
    onDescription={() => (descriptionOpen = true)}
    onRemove={() => (removeOpen = true)}
/>

<ConfirmDialog
    open={removeOpen}
    title={`移除 "${album?.name ?? ''}"?`}
    message="只会删掉本机的登记与索引, 云端文件夹与文件不受影响. 同步到这个相册的本机相册也会一起移除."
    confirmLabel="移除"
    danger
    onClose={() => (removeOpen = false)}
    onConfirm={() => void remove()}
/>

<DescriptionDialog
    bind:open={descriptionOpen}
    bind:draft={descriptionDraft}
    {saving}
    onSave={() => void saveDescription()}
/>

{#if viewerOpen}
    <Viewer items={timeline.items} bind:index={viewerIndex} onClose={() => (viewerOpen = false)} />
{/if}
