<script lang="ts">
    import {getContext} from 'svelte';
    import {rootKey} from '../context';
    import {resolveTabOptions, type NavigationRoot, type TabScreenConfig} from '../navigation.svelte';
    import type {Route, TabOptions} from '../types';

    let {
        screens,
        screenOptions,
        active,
        ariaLabel,
    }: {
        screens: TabScreenConfig[];
        screenOptions: TabOptions | ((route: Route) => TabOptions);
        active: string;
        ariaLabel?: string;
    } = $props();

    const root = getContext<NavigationRoot>(rootKey);
</script>

<nav
    class="z-30 shrink-0 border-t border-line bg-light/95 backdrop-blur-xl"
    aria-label={ariaLabel}
    style:padding-bottom="var(--safe-bottom, env(safe-area-inset-bottom, 0px))"
>
    <div class="mx-auto flex max-w-3xl items-stretch">
        {#each screens as screen (screen.name)}
            {@const options = resolveTabOptions(screen, screenOptions)}
            {@const Icon = options.icon}
            <button
                type="button"
                onclick={() => root.activate(screen.name)}
                class="flex flex-1 flex-col items-center gap-1 py-2 text-[11px] font-medium transition-colors {active ===
                screen.name
                    ? 'text-primary'
                    : 'text-muted'}"
                aria-current={active === screen.name ? 'page' : undefined}
            >
                {#if Icon}
                    <Icon size={23} />
                {/if}
                <span>{options.title ?? screen.name}</span>
            </button>
        {/each}
    </div>
</nav>
