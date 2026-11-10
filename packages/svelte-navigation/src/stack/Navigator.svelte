<script lang="ts" generics="ParamList extends ParamListBase">
    import {getContext, onMount, setContext, type Snippet} from 'svelte';
    import {rootKey, stackKey, tabKey} from '../context';
    import {StackController, type NavigationRoot} from '../navigation.svelte';
    import type {ParamListBase, Route, StackOptions} from '../types';
    import Layer from './Layer.svelte';

    let {
        initialRouteName,
        screenOptions = {},
        children,
    }: {
        initialRouteName: keyof ParamList & string;
        screenOptions?: StackOptions | ((route: Route<ParamList>) => StackOptions);
        children: Snippet;
    } = $props();

    const root = getContext<NavigationRoot>(rootKey);
    const tab = getContext<string>(tabKey) ?? '';
    const controller = new StackController({
        root,
        tab,
        // svelte-ignore state_referenced_locally
        initialRouteName,
        screenOptions: () => screenOptions as StackOptions | ((route: Route<ParamListBase>) => StackOptions),
    });
    setContext(stackKey, controller);

    onMount(() => {
        controller.mount();
        return () => controller.unmount();
    });

    const layers = $derived([
        ...controller.entries.map((entry, index) => ({
            entry,
            top: index === controller.entries.length - 1,
            exiting: false,
        })),
        ...controller.leaving.map((entry) => ({entry, top: false, exiting: true})),
    ]);
</script>

{@render children()}

<div class="relative h-full w-full overflow-hidden">
    {#each layers as layer (layer.entry.key)}
        <Layer {controller} entry={layer.entry} top={layer.top} exiting={layer.exiting} />
    {/each}
</div>
