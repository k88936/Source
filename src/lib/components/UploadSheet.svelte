<script lang="ts">
    import {api} from '#lib/api/client';
    import type {AlbumSummary} from '#lib/api/bindings/AlbumSummary';
    import type {LocalAlbumSummary} from '#lib/api/bindings/LocalAlbumSummary';
    import type {LocalAsset} from '#lib/api/bindings/LocalAsset';
    import {common} from '#lib/i18n/common.i18n';
    import {app} from '#lib/state/app.svelte';
    import {i18n} from '#lib/state/i18n.svelte';
    import AssetThumb from './AssetThumb.svelte';
    import {Image, Upload, X} from '@lucide/svelte';
    import {upload as text} from './UploadSheet.i18n';

    let {
        open = $bindable(false),
        initialDest = $bindable(''),
        initialSelection = [],
        onClose,
        onUploaded,
    }: {
        open?: boolean;
        initialDest?: string;
        initialSelection?: string[];
        onClose?: () => void;
        onUploaded?: () => void;
    } = $props();

    const t = $derived(i18n.pick(text));
    const c = $derived(i18n.pick(common));

    let assets = $state<LocalAsset[]>([]);
    let albums = $state<AlbumSummary[]>([]);
    let localAlbums = $state<LocalAlbumSummary[]>([]);
    let selected = $state<string[]>([]);
    let dest = $state(initialDest);
    let loading = $state(false);
    let uploading = $state(false);
    // `null` means the whole device, otherwise a registered local album's uri.
    let source = $state<string | null>(null);

    const uploadLabel = $derived.by(() => {
        if (uploading) return t.enqueuing;
        if (!dest) return t.needAlbum;
        return t.uploadLabel(selected.length, destName());
    });

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
        selected = selected.includes(asset.id) ? selected.filter((id) => id !== asset.id) : [...selected, asset.id];
    }

    function destName(): string {
        return albums.find((album) => album.id === dest)?.name ?? t.pickAlbum;
    }

    async function upload() {
        if (!selected.length || !dest || uploading) return;
        uploading = true;
        try {
            const tasks = await api.enqueueUploads({
                localIds: selected,
                albumId: dest,
            });
            app.showToast(t.queued(tasks.length, destName()), 'success');
            open = false;
            onUploaded?.();
            onClose?.();
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        } finally {
            uploading = false;
        }
    }

    function close() {
        open = false;
        onClose?.();
    }
</script>

{#if open}
    <div class="fixed inset-0 z-50 flex items-end justify-center bg-black/50" role="dialog" aria-modal="true">
        <div class="flex h-[88dvh] w-full max-w-2xl flex-col rounded-t-3xl bg-light">
            <div class="flex items-center justify-between px-4 pb-2 pt-3">
                <h2 class="text-base font-semibold">{t.title}</h2>
                <button
                    onclick={close}
                    class="grid size-9 place-items-center rounded-full bg-gray text-dark hover:bg-line/40"
                    aria-label={c.close}
                >
                    <X size={18} />
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
                        {c.all}
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
                        <Image size={30} />
                        <p>{t.empty}</p>
                        <p class="text-[11px] text-muted">{t.emptyHint}</p>
                    </div>
                {:else}
                    <div class="grid grid-cols-4 gap-[2px] sm:grid-cols-5">
                        {#each assets as asset (asset.id)}
                            {@const video = asset.mime?.startsWith('video') ?? false}
                            <AssetThumb
                                source={{kind: 'localThumb', uri: asset.uri}}
                                name={asset.name}
                                {video}
                                selectionMode
                                selected={selected.includes(asset.id)}
                                onToggle={() => toggle(asset)}
                            />
                        {/each}
                    </div>
                {/if}
            </div>

            <div class="border-t border-line px-4 pb-[calc(0.75rem+var(--safe-bottom))] pt-3">
                <label class="mb-2 flex items-center gap-2 text-xs text-muted">
                    <span class="shrink-0">{t.destination}</span>
                    <select
                        bind:value={dest}
                        class="min-w-0 flex-1 rounded-lg border border-line bg-transparent px-3 py-1.5 text-xs outline-none focus:ring-2 focus:ring-primary/60"
                    >
                        <option value="">{t.pickAlbum}</option>
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
                    <Upload size={18} />
                    {uploadLabel}
                </button>
            </div>
        </div>
    </div>
{/if}
