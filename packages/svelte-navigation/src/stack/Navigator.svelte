<script lang="ts">
    import {getContext, onMount, setContext, type Snippet} from 'svelte';
    import {rootKey, stackKey, tabKey} from '../context';
    import {StackController, type NavigationRoot, type ScreenEntry} from '../navigation.svelte';
    import type {Animation, ParamListBase, Route, StackHeaderProps, StackScreen, StackOptions} from '../types';
    import Layer from './Layer.svelte';

    // The factory that hands this component out carries the typed face, so the
    // body only deals with names and screens; the param list is checked there.
    let {
        screens,
        initialRouteName,
        screenOptions = {},
        header,
        layerClass = '',
    }: {
        screens: Record<string, StackScreen>;
        initialRouteName: string;
        screenOptions?: StackOptions | ((route: Route<ParamListBase>) => StackOptions);
        header?: Snippet<[StackHeaderProps]>;
        layerClass?: string;
    } = $props();

    const root = getContext<NavigationRoot>(rootKey);
    const tab = getContext<string>(tabKey) ?? '';
    const controller = new StackController({
        root,
        tab,
        // svelte-ignore state_referenced_locally
        initialRouteName,
        screenOptions: () => screenOptions,
    });
    // svelte-ignore state_referenced_locally
    controller.setScreens(screens);
    setContext(stackKey, controller);

    onMount(() => {
        controller.mount();
        return () => controller.unmount();
    });

    const animationOf = (entry: ScreenEntry | undefined): Animation | undefined =>
        entry ? controller.optionsFor(entry).animation : undefined;

    // The layer below waits where the screen above it pushes it, so it needs that
    // screen's animation: a modal sliding up must not drag the stack to the left.
    const layers = $derived([
        ...controller.entries.map((entry, index) => ({
            entry,
            top: index === controller.entries.length - 1,
            exiting: false,
            behindAnimation: animationOf(controller.entries[index + 1]),
        })),
        ...controller.leaving.map((entry) => ({entry, top: false, exiting: true, behindAnimation: undefined})),
    ]);
</script>

<div class="relative h-full w-full overflow-hidden">
    {#each layers as layer (layer.entry.key)}
        <Layer
            {controller}
            entry={layer.entry}
            top={layer.top}
            exiting={layer.exiting}
            behindAnimation={layer.behindAnimation}
            {header}
            {layerClass}
        />
    {/each}
</div>
