<script lang="ts">
    import {onMount} from 'svelte';
    import {useBackHandler, useIsFocused, useNavigation, useScrollContainer} from 'svelte-navigation';
    import {api} from '#lib/api/client';
    import type {AssetSummary} from '#lib/api/bindings/AssetSummary';
    import AssetList from '#lib/components/assets/AssetList/AssetList.svelte';
    import EmptyState from '#lib/components/assets/EmptyState/EmptyState.svelte';
    import {AssetBrowser} from '#lib/components/assets/browser.svelte';
    import {assetsView} from '#lib/api/views';
    import {Image, Search as SearchIcon, X} from '@lucide/svelte';
    import SearchInput from '#lib/components/search/SearchInput/SearchInput.svelte';
    import SearchMode from '#lib/components/search/SearchMode/SearchMode.svelte';
    import SelectAllButton from '#lib/components/selection/SelectAllButton/SelectAllButton.svelte';
    import SelectionOverlay from '#lib/components/selection/SelectionOverlay/SelectionOverlay.svelte';
    import TimelineActions from '#lib/components/viewer/TimelineActions/TimelineActions.svelte';
    import {common} from '#lib/i18n/common.i18n';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import {app} from '#lib/app/app.svelte';
    import {changes} from '#lib/api/changes';
    import ScreenOptions from '#lib/components/shell/ScreenOptions/ScreenOptions.svelte';
    import type {PhotosNavParamList} from './route';
    import {photos} from './Photos.i18n';

    const navigation = useNavigation<PhotosNavParamList>();
    const focused = $derived(useIsFocused());
    const scroll = useScrollContainer();
    const takeBack = useBackHandler();

    const browser = new AssetBrowser(
        {
            window: ({from, to}, listeners) => assetsView.watch({albumId: null, from, to}, listeners),
            edge: (after, depth) => api.assetEdge({albumId: null, after, depth}),
            count: () => api.assetCount({albumId: null}),
        },
        {scroll},
    );

    const t = $derived(i18n.pick(photos));
    const c = $derived(i18n.pick(common));

    $effect(() => {
        // The rows arrive through the window; a listing pass only moves the
        // length the counter prints.
        return changes.subscribe((change) => {
            if (change.kind !== 'albums') return;
            void browser.timeline.refreshTotal();
            if (browser.search.open) void browser.search.run();
        });
    });

    $effect(() => () => browser.dispose());

    onMount(() => {
        void browser.timeline.reload();
    });

    // The back key closes a mode before it leaves the timeline.
    $effect(() => {
        if (!browser.search.open && !browser.selection.active) return;
        return takeBack(() => browser.back());
    });

    function open(asset: AssetSummary) {
        browser.open(asset, navigation, viewerActions);
    }

    function onKeydown(event: KeyboardEvent) {
        // Escape leaves search, and only while this screen is the one on top.
        if (event.key !== 'Escape' || !focused || !browser.search.open || browser.selection.active) return;
        browser.search.exit();
    }
</script>

<svelte:head><title>{c.photos} - Source</title></svelte:head>

<svelte:window onkeydown={onKeydown} />

{#snippet searchContent()}
    <SearchInput search={browser.search} />
{/snippet}

{#snippet actions()}
    {#if browser.selection.active}
        <SelectAllButton selection={browser.selection} ids={browser.shown.items.map((item) => item.id)} />
        <button
            onclick={() => browser.selection.exit()}
            class="grid size-10 place-items-center rounded-full text-dark"
            aria-label={c.exitSelection}
        >
            <X size={21} />
        </button>
    {:else if browser.search.open}
        <button
            onclick={() => browser.search.exit()}
            class="rounded-full px-3 py-2 text-xs font-medium text-muted transition-colors hover:bg-gray"
            aria-label={c.cancel}
        >
            {c.cancel}
        </button>
    {:else}
        <button
            onclick={() => browser.search.enter()}
            class="grid size-9 place-items-center rounded-full text-dark transition-colors hover:bg-gray"
            aria-label={c.search}
        >
            <SearchIcon size={20} />
        </button>
    {/if}
{/snippet}

<!-- The viewer shows what it is given: the timeline injects its own controls. -->
{#snippet viewerActions({index}: {index: number})}
    <TimelineActions timeline={browser.shown} {index} />
{/snippet}

<ScreenOptions
    title={browser.selection.active ? c.selected(browser.selection.count) : c.photos}
    subtitle={browser.selection.active ? '' : (app.account?.siteName ?? '')}
    content={browser.search.open && !browser.selection.active ? searchContent : undefined}
    contentCentered
    {actions}
/>

{#if browser.search.open}
    <SearchMode search={browser.search} selection={browser.selection} onOpen={open} />
{:else if !browser.timeline.loaded}
    <div class="grid gap-[2px]" style:grid-template-columns="repeat({app.columns}, minmax(0, 1fr))">
        {#each Array(app.columns * 6) as _, i (i)}
            <div class="aspect-square animate-pulse bg-gray"></div>
        {/each}
    </div>
{:else if browser.timeline.items.length === 0 && !browser.timeline.loading}
    <EmptyState
        icon={Image}
        title={browser.timeline.error ? t.failed : t.empty}
        description={browser.timeline.error}
        actionLabel={browser.timeline.error ? c.retry : c.addAlbum}
        onAction={() => (browser.timeline.error ? void browser.timeline.reload() : navigation.navigate('Albums'))}
    />
{:else}
    <AssetList
        timeline={browser.timeline}
        selection={browser.selection}
        columns={app.columns}
        onOpen={open}
        onColumns={(columns) => app.setColumns(columns)}
    />
{/if}

<SelectionOverlay selection={browser.selection} />
