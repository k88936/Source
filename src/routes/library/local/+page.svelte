<script lang="ts">
    import {onMount} from 'svelte';
    import {api} from '#lib/api/client';
    import type {AlbumSummary} from '#lib/api/bindings/AlbumSummary';
    import type {LocalAlbumSummary} from '#lib/api/bindings/LocalAlbumSummary';
    import type {LocalEntry} from '#lib/api/bindings/LocalEntry';
    import AlbumPickerSheet from '#lib/components/AlbumPickerSheet.svelte';
    import EmptyState from '#lib/components/EmptyState.svelte';
    import Icon from '#lib/components/Icon.svelte';
    import LocalDirPicker from '#lib/components/LocalDirPicker.svelte';
    import TopBar from '#lib/components/TopBar.svelte';
    import {app} from '#lib/state/app.svelte';

    let albums = $state<LocalAlbumSummary[]>([]);
    let loading = $state(true);
    let pickerOpen = $state(false);
    let targetOpen = $state(false);
    // The directory chosen in step one; step two asks for the album it syncs to.
    let pending = $state<LocalEntry | null>(null);
    let saving = $state(false);

    onMount(() => {
        void load();
    });

    async function load() {
        try {
            albums = await api.localAlbumList();
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        } finally {
            loading = false;
        }
    }

    function chose(entry: LocalEntry) {
        pending = entry;
        targetOpen = true;
    }

    /// A local album only exists to sync somewhere, so registering one and
    /// pointing it at an album are a single step.
    async function register(album: AlbumSummary) {
        const entry = pending;
        if (!entry || saving) return;
        saving = true;
        try {
            const created = await api.addLocalAlbum({
                uri: entry.uri,
                name: entry.name,
                albumId: album.id,
            });
            albums = [...albums.filter((row) => row.id !== created.id), created];
            app.showToast(`"${created.name}" 已同步到 "${created.albumName}"`, 'success');
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        } finally {
            saving = false;
            pending = null;
        }
    }
</script>

<svelte:head><title>本机相册 - Source</title></svelte:head>

<TopBar title="本机相册" subtitle="同步本机目录到云端相册" back>
    {#snippet actions()}
        <button
            onclick={() => (pickerOpen = true)}
            class="grid size-9 place-items-center rounded-full text-dark transition-colors hover:bg-gray"
            aria-label="同步本机相册"
        >
            <Icon name="upload" size={20} />
        </button>
    {/snippet}
</TopBar>

{#if loading}
    <div class="space-y-2 px-4 py-3">
        {#each Array(3) as _}
            <div class="h-16 animate-pulse rounded-2xl bg-gray"></div>
        {/each}
    </div>
{:else if albums.length === 0}
    <EmptyState
        icon="folder"
        title="还没有同步的本机相册"
        description="选一个本机目录, 指向一个云端相册, 它就出现在这里."
        actionLabel="同步本机相册"
        onAction={() => (pickerOpen = true)}
    />
{:else}
    <ul class="space-y-2 px-4 py-3">
        {#each albums as album (album.id)}
            <li>
                <a
                    href="/library/{encodeURIComponent(album.id)}"
                    class="flex items-center gap-3 rounded-2xl bg-gray p-3 transition-colors hover:bg-line/40"
                >
                    <div class="grid size-11 shrink-0 place-items-center rounded-xl bg-primary/10 text-primary">
                        <Icon name="folder" size={22} />
                    </div>
                    <div class="min-w-0 flex-1">
                        <p class="truncate text-sm font-semibold">{album.name}</p>
                        <p class="truncate text-xs text-muted">
                            {album.assetCount} 项 - 同步到 "{album.albumName}"
                        </p>
                    </div>
                    <Icon name="forward" size={18} class="text-muted" />
                </a>
            </li>
        {/each}
    </ul>
    <p class="px-5 pb-24 text-[11px] text-muted">
        自动同步尚未开启: 登记只记录 "这个目录归这个相册", 上传仍从相册里手动发起.
    </p>
{/if}

<LocalDirPicker bind:open={pickerOpen} onPick={chose} />
<AlbumPickerSheet
    bind:open={targetOpen}
    title="同步到哪个相册"
    hint="只记录这个本机目录归哪个相册, 不会移动或上传任何文件."
    onPick={(album) => void register(album)}
/>
