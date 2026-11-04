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
        Trash,
        Upload,
    } from '@lucide/svelte';
    import TopBar from '#lib/components/TopBar.svelte';
    import UploadQueueCard from '#lib/components/UploadQueueCard.svelte';
    import UploadSheet from '#lib/components/UploadSheet.svelte';
    import {app} from '#lib/state/app.svelte';

    let localAlbums = $state<LocalAlbumSummary[]>([]);
    let uploadOpen = $state(false);

    // Entries Immich's library page has and this app does not implement yet.
    // They are listed so the shape of the page is already the final one, and
    // greyed so nothing promises a feature that would do nothing when tapped.
    const upcoming = [
        {icon: Heart, label: '收藏'},
        {icon: LibraryBig, label: '归档'},
        {icon: Cloud, label: '共享链接'},
        {icon: Trash, label: '回收站'},
        {icon: Image, label: '人物'},
        {icon: Calendar, label: '地点'},
        {icon: Play, label: '回忆'},
        {icon: Folder, label: '云端文件夹'},
        {icon: Eye, label: '锁定文件夹'},
        {icon: LayoutGrid, label: '伙伴'},
    ];

    onMount(() => {
        if (page.url.searchParams.get('upload') === '1') uploadOpen = true;
        void loadLocal();
    });

    async function loadLocal() {
        try {
            localAlbums = await api.localAlbumList();
        } catch {
            localAlbums = [];
        }
    }
</script>

<svelte:head><title>资料库 - Source</title></svelte:head>

<TopBar title="资料库" subtitle={app.account?.user.email ?? ''}>
    {#snippet actions()}
        <button
            onclick={() => (uploadOpen = true)}
            class="grid size-9 place-items-center rounded-full text-dark transition-colors hover:bg-gray"
            aria-label="上传"
        >
            <Upload size={20} />
        </button>
    {/snippet}
</TopBar>

<div class="space-y-3 px-4 pb-24 pt-2">
    <CapacityCard />

    <a href="/library/local" class="flex items-center gap-3 rounded-2xl bg-gray p-4 transition-colors hover:bg-line/40">
        <div class="grid size-11 shrink-0 place-items-center rounded-full bg-primary/10 text-primary">
            <Folder size={22} />
        </div>
        <div class="min-w-0 flex-1">
            <p class="text-sm font-semibold">本机相册</p>
            <p class="truncate text-xs text-muted">
                {localAlbums.length === 0
                    ? '把一个本机目录同步到云端相册'
                    : `${localAlbums.length} 个已同步 - ${localAlbums.reduce((sum, album) => sum + album.assetCount, 0)} 张照片`}
            </p>
        </div>
        <ChevronRight size={18} class="text-muted" />
    </a>

    <UploadQueueCard />

    <div>
        <p class="mb-1.5 px-1 text-xs font-medium text-muted">更多</p>
        <div class="grid grid-cols-2 gap-2">
            {#each upcoming as entry (entry.label)}
                {@const Icon = entry.icon}
                <div
                    class="flex items-center gap-2.5 rounded-2xl border border-line px-3 py-3 text-sm text-muted/60"
                    aria-disabled="true"
                    title="尚未开放"
                >
                    <Icon size={18} />
                    <span class="truncate">{entry.label}</span>
                </div>
            {/each}
        </div>
    </div>
</div>

<UploadSheet bind:open={uploadOpen} onUploaded={() => void app.refreshUploads()} />
