<script lang="ts" generics="ParamList extends ParamListBase">
    import {getContext, onDestroy, type Component} from 'svelte';
    import {tabScreensKey} from '../context';
    import type {TabScreenConfig} from '../navigation.svelte';
    import type {ParamListBase, Route, TabOptions} from '../types';

    let {
        name,
        component,
        options,
    }: {
        name: keyof ParamList & string;
        component: Component<Record<string, never>>;
        options?: TabOptions | ((route: Route<ParamList>) => TabOptions);
    } = $props();

    const registry = getContext<{screens: TabScreenConfig[]}>(tabScreensKey);

    // Registration runs during the navigator's render, so the bar already knows this tab.
    // svelte-ignore state_referenced_locally
    registry.screens.push({name, component, options} as TabScreenConfig);

    onDestroy(() => {
        const index = registry.screens.findIndex((screen) => screen.name === name);
        if (index >= 0) registry.screens.splice(index, 1);
    });
</script>
