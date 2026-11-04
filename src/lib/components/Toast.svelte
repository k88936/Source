<script lang="ts">
    import {CircleAlert, CircleCheck, Info} from '@lucide/svelte';
    import {fly} from 'svelte/transition';
    import {app} from '#lib/state/app.svelte';

    const kind = $derived(app.toast?.kind ?? 'info');

    const accents = {
        success: 'bg-success/10 text-success',
        error: 'bg-danger/10 text-danger',
        info: 'bg-info/10 text-info',
    } as const;

    const icons = {
        success: CircleCheck,
        error: CircleAlert,
        info: Info,
    } as const;

    const accent = $derived(accents[kind]);
    const Icon = $derived(icons[kind]);
</script>

{#if app.toast}
    <div class="pointer-events-none fixed inset-x-0 bottom-24 z-[60] flex justify-center px-4">
        <div
            transition:fly={{y: 10, duration: 160}}
            class="flex max-w-sm items-center gap-3 rounded-2xl bg-gray py-2.5 pe-4 ps-2.5 text-sm text-dark shadow-lg ring-1 ring-line"
        >
            <span class="grid size-7 shrink-0 place-items-center rounded-full {accent}">
                <Icon size={16} />
            </span>
            <span class="leading-snug">{app.toast.message}</span>
        </div>
    </div>
{/if}
