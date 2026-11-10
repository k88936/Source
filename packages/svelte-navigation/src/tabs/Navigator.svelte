<script lang="ts">
    import {getContext, setContext, type Snippet} from 'svelte';
    import {rootKey, tabScreensKey} from '../context';
    import type {NavigationRoot, TabScreenConfig} from '../navigation.svelte';
    import type {TabBarProps} from '../types';
    import Content from './Content.svelte';

    let {
        tabBar,
        children,
    }: {
        tabBar: Snippet<[TabBarProps]>;
        children: Snippet;
    } = $props();

    const root = getContext<NavigationRoot>(rootKey);
    const registry = $state<{screens: TabScreenConfig[]}>({screens: []});
    setContext(tabScreensKey, registry);

    let mounted = $state<string[]>([]);

    const active = $derived(root.active || registry.screens[0]?.name || '');

    $effect(() => {
        root.setTabs(registry.screens.map((screen) => screen.name));
        const name = root.active || registry.screens[0]?.name;
        if (name && !mounted.includes(name)) mounted.push(name);
    });
</script>

{@render children()}

<div class="flex h-full flex-col">
    <div class="relative min-h-0 flex-1">
        {#each registry.screens as screen (screen.name)}
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
        tabs: registry.screens.map((screen) => screen.name),
        active,
        activate: (name) => root.activate(name),
    })}
</div>
