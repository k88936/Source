<script lang="ts">
    import {onMount} from 'svelte';
    import {page} from '$app/state';
    import {api} from '#lib/api/client';
    import type {LocalAlbumSummary} from '#lib/api/bindings/LocalAlbumSummary';
    import CapacityCard from '#lib/components/CapacityCard.svelte';
    import {
        Calendar,
        ChevronRight,
        Cloud,
        Eye,
        Folder,
        Heart,
        Image,
        LayoutGrid,
        LibraryBig,
        Play,
        RefreshCw,
        Square,
        Trash,
    } from '@lucide/svelte';
    import TopBar from '#lib/components/TopBar.svelte';
    import UploadQueueCard from '#lib/components/UploadQueueCard.svelte';
    import {common} from '#lib/i18n/common.i18n';
    import {app} from '#lib/state/app.svelte';
    import {i18n} from '#lib/state/i18n.svelte';
    import {library as text} from './page.i18n';

    let localAlbums = $state<LocalAlbumSummary[]>([]);

    const t = $derived(i18n.pick(text));
    const c = $derived(i18n.pick(common));
    // One directory is one job, so the button is busy while any directory of
    // this kind is still queued, paused or being walked.
    const syncing = $derived(
        app.jobs.some(
            (job) =>
                job.key === 'sync' && (job.state === 'queued' || job.state === 'running' || job.state === 'paused'),
        ),
    );

    // Entries Immich's library page has and this app does not implement yet.
    // They are listed so the shape of the page is already the final one, and
    // greyed so nothing promises a feature that would do nothing when tapped.
    const upcoming = $derived([
        {icon: Heart, label: t.upcoming.favorites},
        {icon: LibraryBig, label: t.upcoming.archive},
        {icon: Cloud, label: t.upcoming.sharedLinks},
        {icon: Trash, label: t.upcoming.trash},
        {icon: Image, label: t.upcoming.people},
        {icon: Calendar, label: t.upcoming.places},
        {icon: Play, label: t.upcoming.memories},
        {icon: Folder, label: t.upcoming.cloudFolders},
        {icon: Eye, label: t.upcoming.lockedFolders},
        {icon: LayoutGrid, label: t.upcoming.partners},
    ]);

    async function loadLocal() {
        try {
            localAlbums = await api.localAlbumList();
        } catch {
            localAlbums = [];
        }
    }

    async function toggleSync() {
        try {
            if (syncing) {
                await api.syncStop();
                return;
            }
            await api.syncStart();
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        }
    }
</script>

<svelte:head><title>{t.title} - Source</title></svelte:head>

<TopBar title={t.title} subtitle={app.account?.user.email ?? ''}></TopBar>

<div class="space-y-3 px-4 pb-24 pt-2">
    <CapacityCard />

    <div class="flex items-center gap-3 rounded-2xl bg-gray p-4">
        <a href="/library/local" class="flex min-w-0 flex-1 items-center gap-3 transition-opacity hover:opacity-80">
            <div class="grid size-11 shrink-0 place-items-center rounded-full bg-primary/10 text-primary">
                <Folder size={22} />
            </div>
            <div class="min-w-0 flex-1">
                <p class="text-sm font-semibold">{t.localAlbums}</p>
                {#if localAlbums.length > 0}
                    <p class="truncate text-xs text-muted">
                        {t.synced(
                            localAlbums.length,
                            localAlbums.reduce((sum, album) => sum + album.assetCount, 0),
                        )}
                    </p>
                {/if}
            </div>
            <ChevronRight size={18} class="text-muted" />
        </a>
        <button
            onclick={() => void toggleSync()}
            class="grid size-9 shrink-0 place-items-center rounded-full text-dark transition-colors hover:bg-line/60"
            aria-label={syncing ? t.stopSync : t.sync}
        >
            {#if syncing}
                <Square size={16} />
            {:else}
                <RefreshCw size={18} />
            {/if}
        </button>
    </div>

    <UploadQueueCard />

    <div>
        <p class="mb-1.5 px-1 text-xs font-medium text-muted">{t.more}</p>
        <div class="grid grid-cols-2 gap-2">
            {#each upcoming as entry (entry.label)}
                {@const Icon = entry.icon}
                <div
                    class="flex items-center gap-2.5 rounded-2xl border border-line px-3 py-3 text-sm text-muted/60"
                    aria-disabled="true"
                    title={t.notOpen}
                >
                    <Icon size={18} />
                    <span class="truncate">{entry.label}</span>
                </div>
            {/each}
        </div>
    </div>
</div>
