<script lang="ts">
    import {Icon} from '@immich/ui';
    import {mdiImageMultipleOutline, mdiImageOutline, mdiViewGridOutline} from '@mdi/js';
    import type {TabBarProps} from 'svelte-navigation';
    import {common} from '#lib/i18n/common.i18n';
    import {i18n} from '#lib/i18n/i18n.svelte';

    let {tabs, active, activate}: TabBarProps = $props();

    const c = $derived(i18n.pick(common));

    const LABELS = $derived<Record<string, string>>({
        Photos: c.photos,
        Albums: c.albums,
        Library: c.library,
    });

    const ICONS: Record<string, string> = {
        Photos: mdiImageOutline,
        Albums: mdiImageMultipleOutline,
        Library: mdiViewGridOutline,
    };
</script>

<nav
    class="z-30 shrink-0 border-t border-line bg-light/95 backdrop-blur-xl"
    style:padding-bottom="var(--safe-bottom, env(safe-area-inset-bottom, 0px))"
>
    <div class="mx-auto flex max-w-3xl items-stretch">
        {#each tabs as tab (tab)}
            <button
                type="button"
                onclick={() => activate(tab)}
                class="flex flex-1 flex-col items-center gap-1 py-2 text-[11px] font-medium transition-colors {active ===
                tab
                    ? 'text-primary'
                    : 'text-muted'}"
                aria-current={active === tab ? 'page' : undefined}
            >
                {#if ICONS[tab]}
                    <Icon icon={ICONS[tab]} size="23" aria-hidden />
                {/if}
                <span>{LABELS[tab] ?? tab}</span>
            </button>
        {/each}
    </div>
</nav>
