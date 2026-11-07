<script lang="ts">
    import {resolve} from '$app/paths';
    import {api} from '#lib/api/client';
    import type {Capacity} from '#lib/api/bindings/Capacity';
    import {Settings} from '@lucide/svelte';
    import {formatBytes} from '#lib/format';
    import {app} from '#lib/state/app.svelte';
    import {i18n} from '#lib/state/i18n.svelte';
    import {capacity as text} from './CapacityCard.i18n';

    const t = $derived(i18n.pick(text));

    let capacity = $state<Capacity | null>(null);

    // The quota is the one account fact that changes on its own, so this card
    // asks the server for it instead of reading a cached copy of the account.
    // It asks again once the stored session turns live: at start-up no client
    // stands behind the account yet, and a refusal then would leave the card
    // on "unknown" for the rest of the process.
    $effect(() => {
        void app.account?.user.id;
        if (app.setup !== 'live') return;
        let live = true;
        void api
            .capacity()
            .then((next) => {
                if (live) capacity = next;
            })
            .catch(() => {
                // capacity is informational
            });
        return () => {
            live = false;
        };
    });

    const usedPercent = $derived.by(() => {
        if (!capacity || capacity.total <= 0) return 0;
        return Math.min(100, Math.round((capacity.used / capacity.total) * 100));
    });
</script>

<a href={resolve('/settings')} class="block rounded-2xl bg-gray p-4 transition-colors hover:bg-line/40">
    <div class="flex items-center gap-3">
        <div
            class="grid size-12 shrink-0 place-items-center rounded-full bg-primary text-lg font-semibold text-on-primary"
        >
            {app.account?.user.name?.slice(0, 1).toUpperCase() ?? '?'}
        </div>
        <div class="min-w-0 flex-1">
            <p class="truncate text-base font-semibold">{app.account?.user.name}</p>
            <p class="truncate text-xs text-muted">
                {app.account?.siteName ?? app.account?.server}
            </p>
        </div>
        <Settings size={19} class="text-muted" />
    </div>

    <div class="mt-4">
        <div class="mb-1 flex justify-between text-[11px] text-muted">
            <span>{t.storage}</span>
            <span>
                {#if capacity}
                    {formatBytes(capacity.used)} / {formatBytes(capacity.total)}
                {:else}
                    {t.unknown}
                {/if}
            </span>
        </div>
        <div class="h-1.5 overflow-hidden rounded-full bg-muted/25">
            <div class="h-full rounded-full bg-primary" style:width="{usedPercent}%"></div>
        </div>
    </div>
</a>
