<script lang="ts">
    import {useNavigation} from 'svelte-navigation';
    import CapacityCard from './components/CapacityCard/CapacityCard.svelte';
    import {
        Calendar,
        Cloud,
        Eye,
        Folder,
        Heart,
        Image,
        LayoutGrid,
        LibraryBig,
        Play,
        Trash,
    } from '@lucide/svelte';
    import {common} from '#lib/i18n/common.i18n';
    import {app} from '#lib/app/app.svelte';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import type {LibraryNavParamList} from './route';
    import ScreenOptions from '#lib/components/shell/ScreenOptions/ScreenOptions.svelte';
    import {library as text} from './Library.i18n';

    const navigation = useNavigation<LibraryNavParamList>();

    const t = $derived(i18n.pick(text));
    const c = $derived(i18n.pick(common));

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
</script>

<svelte:head><title>{c.library} - Source</title></svelte:head>

<ScreenOptions title={c.library} subtitle={app.account?.user.email ?? ''} />

<div class="space-y-3 px-4 pb-24 pt-2">
    <CapacityCard onOpen={() => navigation.navigate('Settings')} />

    <div>
        <p class="mb-1.5 px-1 text-xs font-medium text-muted">{c.more}</p>
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
