<script lang="ts">
    import {common} from '#lib/i18n/common.i18n';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import {description} from './DescriptionDialog.i18n';
    import Overlay from '../Overlay/Overlay.svelte';

    let {
        open = $bindable(false),
        draft = $bindable(''),
        saving = false,
        onSave,
    }: {
        open?: boolean;
        draft?: string;
        saving?: boolean;
        onSave: () => void;
    } = $props();

    const t = $derived(i18n.pick(description));
    const c = $derived(i18n.pick(common));
</script>

<Overlay bind:open label={t.title}>
    <div class="pointer-events-none flex h-full items-center justify-center p-4">
        <div class="pointer-events-auto w-full max-w-md rounded-3xl bg-light p-5 shadow-2xl">
            <h2 class="text-lg font-medium">{t.title}</h2>
            <textarea bind:value={draft} rows="4" placeholder={t.placeholder} class="field mt-3 resize-none"></textarea>
            <div class="mt-4 flex justify-end gap-2">
                <button
                    onclick={() => (open = false)}
                    class="rounded-xl px-4 py-2 text-sm font-medium text-muted hover:bg-gray"
                >
                    {c.cancel}
                </button>
                <button
                    onclick={onSave}
                    disabled={saving}
                    class="rounded-xl bg-primary px-4 py-2 text-sm font-medium text-on-primary transition-opacity hover:opacity-90 disabled:opacity-50"
                >
                    {saving ? t.saving : c.save}
                </button>
            </div>
        </div>
    </div>
</Overlay>
