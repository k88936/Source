<script lang="ts">
    import {onMount, setContext, type Snippet} from 'svelte';
    import {screenKey, scrollKey} from '../context';
    import type {ScreenEntry, StackController} from '../navigation.svelte';
    import type {Animation, StackHeaderProps} from '../types';
    import {behindPlacement, enterPlacement, transitionOf} from './transition';

    let {
        controller,
        entry,
        top,
        exiting = false,
        behindAnimation,
        header,
        layerClass = '',
    }: {
        controller: StackController;
        entry: ScreenEntry;
        top: boolean;
        exiting?: boolean;
        /** Animation of the screen above this one: it decides how this layer is pushed. */
        behindAnimation?: Animation;
        header?: Snippet<[StackHeaderProps]>;
        layerClass?: string;
    } = $props();

    // svelte-ignore state_referenced_locally
    setContext(screenKey, entry);

    const options = $derived(controller.optionsFor(entry));
    const config = $derived(controller.configs[entry.name]);
    const animation = $derived(options.animation ?? 'slide_from_right');
    const modal = $derived(options.presentation === 'modal');
    // Only a layer with something below it can go back: the stack root keeps its own
    // entry while a child is pushed, and must not grow a second back arrow.
    const canBack = $derived(controller.entries[0] !== entry && !controller.root.hasMode(entry));

    // A screen that is already there when the app opens has nowhere to slide from.
    // svelte-ignore state_referenced_locally
    let entered = $state(!entry.animate);
    onMount(() => {
        if (entered) return;
        requestAnimationFrame(() => (entered = true));
    });

    // A layer moves either as itself (it enters, leaves or sits on top) or as the
    // screen above pushes it out of the way.
    const motion = $derived(top || exiting ? animation : (behindAnimation ?? animation));

    const placement = $derived.by(() => {
        if (!entered || exiting) return enterPlacement(animation);
        if (top) return {transform: 'none', opacity: '1'};
        return behindPlacement(motion);
    });

    const style = $derived(
        [
            `transform:${placement.transform}`,
            `opacity:${placement.opacity}`,
            `transition:${entered ? transitionOf(motion) : 'none'}`,
            `z-index:${exiting ? 30 : modal ? 40 : top ? 20 : 10}`,
            `pointer-events:${top && !exiting ? 'auto' : 'none'}`,
        ].join(';'),
    );

    let scroll = $state<HTMLElement>();
    setContext(scrollKey, () => scroll);
</script>

<div class="{modal ? 'fixed inset-0' : 'absolute inset-0'} {layerClass} flex flex-col" {style}>
    {#if options.headerShown !== false && header}
        {@render header({options, canBack, back: () => controller.root.back(controller)})}
    {/if}
    <div class="min-h-0 flex-1 overflow-y-auto" bind:this={scroll}>
        {#if config}
            {@const Component = config.component}
            <Component />
        {/if}
    </div>
</div>
