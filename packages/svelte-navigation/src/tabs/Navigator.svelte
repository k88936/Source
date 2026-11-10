<script lang="ts">
    import {getContext, onDestroy, type Snippet} from 'svelte';
    import {rootKey} from '../context';
    import type {NavigationRoot, TabScreenConfig} from '../navigation.svelte';
    import type {TabBarProps, TabScreen} from '../types';
    import Content from './Content.svelte';

    let {
        tabs,
        tabBar,
    }: {
        tabs: Record<string, TabScreen>;
        tabBar: Snippet<[TabBarProps]>;
    } = $props();

    const root = getContext<NavigationRoot>(rootKey);

    // The navigator owns the table, so this happens in one place: the tabs, their
    // order, and the paths of a tab whose stack has not been mounted yet.
    // svelte-ignore state_referenced_locally
    const screens: TabScreenConfig[] = Object.entries(tabs).map(([name, tab]) => ({
        name,
        component: tab.component,
    }));
    // svelte-ignore state_referenced_locally
    for (const [name, tab] of Object.entries(tabs)) root.registerPatterns(name, tab.screens);
    root.setTabs(screens.map((screen) => screen.name));

    let mounted = $state<string[]>([]);

    const active = $derived(root.active || screens[0]?.name || '');

    $effect(() => {
        const name = root.active || screens[0]?.name;
        if (name && !mounted.includes(name)) mounted.push(name);
    });

    onDestroy(() => {
        for (const screen of screens) root.unregisterPatterns(screen.name);
    });
</script>

<div class="flex h-full flex-col">
    <div class="relative min-h-0 flex-1">
        {#each screens as screen (screen.name)}
            <!-- Only mounted once visited, then kept: a tab keeps its stack, its scroll and its data.
                 A hidden tab is not painted at all, so nothing of it can linger on the tab
                 that is being shown instead. -->
            <div
                class="absolute inset-0"
                class:invisible={active !== screen.name}
                style:contain={active === screen.name ? undefined : 'paint'}
            >
                {#if screen.name === active || mounted.includes(screen.name)}
                    <Content name={screen.name} View={screen.component} />
                {/if}
            </div>
        {/each}
    </div>
    {@render tabBar({
        tabs: screens.map((screen) => screen.name),
        active,
        activate: (name) => root.activate(name),
    })}
</div>
