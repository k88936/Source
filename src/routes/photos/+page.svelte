<script lang="ts">
    import {onMount} from 'svelte';
    import {goto} from '$app/navigation';
    import {api} from '#lib/api/client';
    import type {AlbumSummary} from '#lib/api/bindings/AlbumSummary';
    import type {AssetSummary} from '#lib/api/bindings/AssetSummary';
    import AlbumPickerSheet from '#lib/components/AlbumPickerSheet.svelte';
    import AssetGrid from '#lib/components/AssetGrid.svelte';
    import EmptyState from '#lib/components/EmptyState.svelte';
    import {Image, SquareDashed, X} from '@lucide/svelte';
    import SelectAllButton from '#lib/components/SelectAllButton.svelte';
    import SelectionBar from '#lib/components/SelectionBar.svelte';
    import TopBar from '#lib/components/TopBar.svelte';
    import Viewer from '#lib/components/Viewer.svelte';
    import {app} from '#lib/state/app.svelte';
    import {changes} from '#lib/state/changes';
    import {Selection} from '#lib/state/selection.svelte';
    import {Timeline} from '#lib/state/timeline.svelte';

    const timeline = new Timeline((cursor, limit) => api.timelinePage({cursor, limit}));
    const selection = new Selection();

    let viewerOpen = $state(false);
    let viewerIndex = $state(0);
    let sentinel: HTMLDivElement | undefined = $state();
    let pickerOpen = $state(false);
    let moving = $state(false);

    $effect(() => {
        // Patch on writes, refetch only when a listing pass moved the ground.
        return changes.subscribe((change) => {
            if (change.kind === 'albums') {
                void timeline.reload();
                return;
            }
            for (const asset of change.assets) timeline.upsert(asset);
        });
    });

    $effect(() => {
        if (!sentinel || !timeline.next) return;
        const observer = new IntersectionObserver(
            (entries) => {
                if (entries[0]?.isIntersecting) void timeline.loadMore();
            },
            {rootMargin: '800px 0px'},
        );
        observer.observe(sentinel);
        return () => observer.disconnect();
    });

    onMount(() => {
        void timeline.reload();
    });

    function open(asset: AssetSummary) {
        const index = timeline.items.findIndex((item) => item.id === asset.id);
        if (index >= 0) {
            viewerIndex = index;
            viewerOpen = true;
        }
    }

    async function downloadSelected() {
        if (selection.count === 0) return;
        try {
            const result = await api.exportAssets({assetIds: selection.ids});
            app.showToast(`已导出 ${result.exported} 项`, 'success');
            selection.exit();
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        }
    }

    async function moveToAlbum(album: AlbumSummary) {
        moving = true;
        try {
            const moved = await api.moveAssetsToAlbum({
                assetIds: [...selection.ids],
                albumId: album.id,
            });
            for (const asset of moved) timeline.upsert(asset);
            app.showToast(`已移动 ${moved.length} 项到 "${album.name}"`, 'success');
            selection.exit();
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        } finally {
            moving = false;
        }
    }
</script>

<svelte:head><title>照片 - Source</title></svelte:head>

<TopBar
    title={selection.active ? `已选择 ${selection.count} 项` : '照片'}
    subtitle={selection.active ? '' : (app.account?.siteName ?? '')}
>
    {#snippet actions()}
        {#if selection.active}
            <SelectAllButton {selection} ids={timeline.items.map((item) => item.id)} />
            <button
                onclick={() => selection.exit()}
                class="grid size-10 place-items-center rounded-full text-dark"
                aria-label="退出选择"
            >
                <X size={21} />
            </button>
        {:else}
            <button
                onclick={() => selection.start()}
                class="grid size-9 place-items-center rounded-full text-dark transition-colors hover:bg-gray"
                aria-label="选择"
            >
                <SquareDashed size={20} />
            </button>
        {/if}
    {/snippet}
</TopBar>

{#if !timeline.loaded}
    <div class="grid gap-[2px]" style:grid-template-columns="repeat({app.columns}, minmax(0, 1fr))">
        {#each Array(app.columns * 6) as _}
            <div class="aspect-square animate-pulse bg-gray"></div>
        {/each}
    </div>
{:else if timeline.items.length === 0 && !timeline.loading}
    <EmptyState
        icon={Image}
        title={timeline.error ? '加载失败' : '时间线还是空的'}
        description={timeline.error}
        actionLabel={timeline.error ? '重试' : '添加相册'}
        onAction={() => (timeline.error ? void timeline.reload() : void goto('/albums'))}
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
        <div bind:this={sentinel} class="flex justify-center pb-24 pt-2">
            <div class="size-6 animate-spin rounded-full border-2 border-line border-t-primary"></div>
        </div>
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

<AlbumPickerSheet bind:open={pickerOpen} onPick={(album) => void moveToAlbum(album)} />

{#if viewerOpen}
    <Viewer items={timeline.items} bind:index={viewerIndex} onClose={() => (viewerOpen = false)} />
{/if}
