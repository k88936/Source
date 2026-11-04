<script lang="ts">
    import {api} from '#lib/api/client';
    import type {AlbumSummary} from '#lib/api/bindings/AlbumSummary';
    import {app} from '#lib/state/app.svelte';
    import {X} from '@lucide/svelte';
    import MediaImage from './MediaImage.svelte';

    let {
        open = $bindable(false),
        excludeId = '',
        title = '移动到相册',
        hint = 'Cloudreve 的相册就是文件夹, 移动后文件会离开原目录.',
        onPick,
    }: {
        open?: boolean;
        /** Album the assets already live in; it is hidden from the list. */
        excludeId?: string;
        /** The sheet is also used to pick a sync target, where nothing moves. */
        title?: string;
        hint?: string;
        onPick: (album: AlbumSummary) => void;
    } = $props();

    let albums = $state<AlbumSummary[]>([]);
    let loading = $state(false);
    let filter = $state('');

    let visible = $derived(
        albums.filter(
            (album) =>
                album.id !== excludeId &&
                (!filter.trim() || album.name.toLowerCase().includes(filter.trim().toLowerCase())),
        ),
    );

    $effect(() => {
        if (!open) return;
        filter = '';
        loading = true;
        api.albumList()
            .then((list) => {
                albums = list;
            })
            .catch((err) => app.showToast(app.describeError(err), 'error'))
            .finally(() => {
                loading = false;
            });
    });
</script>

{#if open}
    <div
        class="fixed inset-0 z-50 flex items-end justify-center bg-black/45 sm:items-center sm:p-6"
        role="dialog"
        aria-modal="true"
        tabindex="-1"
        onclick={(event) => {
            if (event.target === event.currentTarget) open = false;
        }}
        onkeydown={(event) => {
            if (event.key === 'Escape') open = false;
        }}
    >
        <div class="flex max-h-[80vh] w-full max-w-md flex-col rounded-t-3xl bg-light p-4 shadow-2xl sm:rounded-3xl">
            <div class="mb-2 flex items-center justify-between">
                <h2 class="text-base font-semibold">{title}</h2>
                <button onclick={() => (open = false)} class="rounded-full p-2 hover:bg-gray" aria-label="关闭">
                    <X size={20} />
                </button>
            </div>
            <p class="mb-3 text-xs text-muted">{hint}</p>
            <input bind:value={filter} type="search" placeholder="筛选相册" class="field mb-2 h-9" />
            <div class="min-h-0 flex-1 overflow-y-auto">
                {#if loading}
                    <div class="flex justify-center py-8">
                        <div class="size-5 animate-spin rounded-full border-2 border-line border-t-primary"></div>
                    </div>
                {:else if visible.length === 0}
                    <p class="py-8 text-center text-sm text-muted">没有可用的相册</p>
                {:else}
                    <ul class="flex flex-col">
                        {#each visible as album (album.id)}
                            <li>
                                <button
                                    onclick={() => {
                                        open = false;
                                        onPick(album);
                                    }}
                                    class="flex w-full items-center gap-3 rounded-xl px-2 py-2 text-left hover:bg-gray"
                                >
                                    <span class="size-10 shrink-0 overflow-hidden rounded-lg bg-gray">
                                        {#if album.coverAssetIds[0]}
                                            <MediaImage
                                                source={{kind: 'thumb', assetId: album.coverAssetIds[0]}}
                                                alt=""
                                                loading="lazy"
                                                class="size-full object-cover"
                                            />
                                        {/if}
                                    </span>
                                    <span class="min-w-0 flex-1">
                                        <span class="block truncate text-sm font-medium">{album.name}</span>
                                        <span class="block truncate text-[11px] text-muted">{album.assetCount} 项</span>
                                    </span>
                                </button>
                            </li>
                        {/each}
                    </ul>
                {/if}
            </div>
        </div>
    </div>
{/if}
