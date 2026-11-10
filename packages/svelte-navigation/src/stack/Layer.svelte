<script lang="ts">
    import {onMount, setContext} from 'svelte';
    import {screenKey, scrollKey} from '../context';
    import type {ScreenEntry, StackController} from '../navigation.svelte';
    import Header from './Header.svelte';
    import {behindPlacement, enterPlacement, transitionOf} from './transition';

    let {
        controller,
        entry,
        top,
        exiting = false,
    }: {
        controller: StackController;
        entry: ScreenEntry;
        top: boolean;
        exiting?: boolean;
    } = $props();

    // svelte-ignore state_referenced_locally
    setContext(screenKey, entry);

    const options = $derived(controller.optionsFor(entry));
    const config = $derived(controller.configs[entry.name]);
    const animation = $derived(options.animation ?? 'slide_from_right');
    const modal = $derived(options.presentation === 'modal');

    // A screen that is already there when the app opens has nowhere to slide from.
    // svelte-ignore state_referenced_locally
    let entered = $state(!entry.animate);
    onMount(() => {
        if (entered) return;
        requestAnimationFrame(() => (entered = true));
    });

    const placement = $derived.by(() => {
        if (!entered || exiting) return enterPlacement(animation);
        if (top) return {transform: 'none', opacity: '1'};
        return behindPlacement(animation);
    });

    const style = $derived(
        [
            `transform:${placement.transform}`,
            `opacity:${placement.opacity}`,
            `transition:${entered ? transitionOf(animation) : 'none'}`,
            `z-index:${exiting ? 30 : modal ? 40 : top ? 20 : 10}`,
            `pointer-events:${top && !exiting ? 'auto' : 'none'}`,
        ].join(';'),
    );

    let scroll = $state<HTMLElement>();
    setContext(scrollKey, () => scroll);
</script>

<div
    class="{modal
        ? 'fixed inset-0'
        : 'absolute inset-0'} flex flex-col bg-light text-dark"
    {style}
>
    {#if options.headerShown !== false}
        <Header {controller} {entry} {options} />
    {/if}
    <div class="min-h-0 flex-1 overflow-y-auto" bind:this={scroll}>
        {#if config}
            {@const Component = config.component}
            <Component />
        {/if}
    </div>
</div>
