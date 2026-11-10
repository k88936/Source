<script lang="ts">
    import type {AssetDetail} from '#lib/api/bindings/AssetDetail';
    import type {AssetSummary} from '#lib/api/bindings/AssetSummary';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import {X} from '@lucide/svelte';
    import {info} from './AssetInfoSheet.i18n';

    let {
        open = $bindable(false),
        asset,
        detail,
    }: {
        open?: boolean;
        asset: AssetSummary | undefined;
        detail: AssetDetail | null;
    } = $props();

    const t = $derived(i18n.pick(info));

    const dateFormatter = $derived(
        new Intl.DateTimeFormat(i18n.tag, {
            year: 'numeric',
            month: 'long',
            day: 'numeric',
            hour: '2-digit',
            minute: '2-digit',
        }),
    );

    function formatDate(ts: number): string {
        return dateFormatter.format(new Date(ts));
    }
</script>

{#if open}
    <div
        class="absolute inset-x-0 bottom-0 z-10 max-h-[70%] overflow-y-auto rounded-t-3xl bg-gray/95 p-5 pb-[calc(1.25rem+var(--safe-bottom))] text-dark backdrop-blur-xl"
    >
        <div class="mb-3 flex items-center justify-between">
            <div>
                <p class="text-sm font-semibold">{t.details}</p>
                <p class="text-xs text-muted">{detail?.albumPath ?? ''}</p>
            </div>
            <button
                class="grid size-9 place-items-center rounded-full hover:bg-line/40"
                onclick={() => (open = false)}
                aria-label={t.close}
            >
                <X size={18} />
            </button>
        </div>
        {#if asset}
            <dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 text-xs">
                <dt class="text-muted">{t.name}</dt>
                <dd class="truncate text-right">{asset.name}</dd>
                <dt class="text-muted">{t.taken}</dt>
                <dd class="text-right">{formatDate(asset.takenAt)}</dd>
                <dt class="text-muted">{t.size}</dt>
                <dd class="text-right">{(asset.size / 1024 / 1024).toFixed(1)} MB</dd>
                <dt class="text-muted">{t.dimensions}</dt>
                <dd class="text-right">{asset.width ?? '?'} x {asset.height ?? '?'}</dd>
                {#each detail?.exif ?? [] as field (field.key)}
                    <dt class="truncate text-muted" title={field.key}>
                        {field.label}
                    </dt>
                    <dd class="break-all text-right">{field.value}</dd>
                {/each}
            </dl>
        {/if}
    </div>
{/if}
