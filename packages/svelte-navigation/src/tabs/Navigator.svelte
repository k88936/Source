<script lang="ts" generics="ParamList extends ParamListBase">
    import {getContext, setContext, type Snippet} from 'svelte';
    import {rootKey, tabScreensKey} from '../context';
    import type {NavigationRoot, TabScreenConfig} from '../navigation.svelte';
    import type {ParamListBase, Route, TabOptions} from '../types';
    import Bar from './Bar.svelte';
    import Content from './Content.svelte';

    let {
        screenOptions = {},
        ariaLabel,
        children,
    }: {
        screenOptions?: TabOptions | ((route: Route<ParamList>) => TabOptions);
        ariaLabel?: string;
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

<div class="flex h-full flex-col bg-light text-dark">
    <div class="relative min-h-0 flex-1">
        {#each registry.screens as screen (screen.name)}
            <!-- Only mounted once visited, then kept: a tab keeps its stack, its scroll and its data. -->
            <div class="absolute inset-0" class:invisible={active !== screen.name}>
                {#if screen.name === active || mounted.includes(screen.name)}
                    <Content name={screen.name} View={screen.component} />
                {/if}
            </div>
        {/each}
    </div>
    <Bar screens={registry.screens} {active} {ariaLabel} screenOptions={screenOptions as TabOptions | ((route: Route) => TabOptions)} />
</div>
