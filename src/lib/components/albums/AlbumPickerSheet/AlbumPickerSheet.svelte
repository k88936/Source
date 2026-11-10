<script lang="ts">
    import {api} from '#lib/api/client';
    import type {AlbumSummary} from '#lib/api/bindings/AlbumSummary';
    import {common} from '#lib/i18n/common.i18n';
    import {app} from '#lib/app/app.svelte';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import {X} from '@lucide/svelte';
    import MediaImage from '../../assets/MediaImage/MediaImage.svelte';
    import Sheet from '../../overlay/Sheet/Sheet.svelte';
    import {picker} from './AlbumPickerSheet.i18n';

    let {
        open = $bindable(false),
        excludeId = '',
        title,
        onPick,
    }: {
        open?: boolean;
        /** Album the assets already live in; it is hidden from the list. */
        excludeId?: string;
        /** The sheet is also used to pick a sync target, where nothing moves. */
        title?: string;
        onPick: (album: AlbumSummary) => void;
    } = $props();

    const t = $derived(i18n.pick(picker));
    const c = $derived(i18n.pick(common));

    const heading = $derived(title ?? c.moveToAlbum);

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

<Sheet bind:open label={heading} panelClass="max-h-[80vh] max-w-md">
    <div class="flex min-h-0 flex-1 flex-col p-4">
        <div class="mb-2 flex items-center justify-between">
            <h2 class="text-base font-semibold">{heading}</h2>
            <button onclick={() => (open = false)} class="rounded-full p-2 hover:bg-gray" aria-label={c.close}>
                <X size={20} />
            </button>
        </div>
        <input bind:value={filter} type="search" placeholder={t.filter} class="field mb-2 h-9" />
        <div class="min-h-0 flex-1 overflow-y-auto">
            {#if loading}
                <div class="flex justify-center py-8">
                    <div class="size-5 animate-spin rounded-full border-2 border-line border-t-primary"></div>
                </div>
            {:else if visible.length === 0}
                <p class="py-8 text-center text-sm text-muted">{t.empty}</p>
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
                                    <span class="block truncate text-[11px] text-muted"
                                        >{t.items(album.assetCount)}</span
                                    >
                                </span>
                            </button>
                        </li>
                    {/each}
                </ul>
            {/if}
        </div>
    </div>
</Sheet>
