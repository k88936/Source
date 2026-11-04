<script lang="ts">
    import Icon from './Icon.svelte';

    let {
        icon = 'image',
        title,
        description = '',
        actionLabel = '',
        onAction,
    }: {
        icon?: 'image' | 'albums' | 'search' | 'cloud' | 'folder' | 'upload' | 'wifi-off';
        title: string;
        description?: string;
        actionLabel?: string;
        onAction?: () => void;
    } = $props();
</script>

<div class="flex flex-col items-center justify-center px-8 py-20 text-center">
    <div class="flex max-w-sm flex-col items-center gap-4 rounded-3xl bg-gray px-8 py-10">
        <div class="grid size-20 place-items-center rounded-full bg-light text-muted">
            <Icon name={icon} size={34} />
        </div>
        <div>
            <h2 class="text-xl font-medium">{title}</h2>
            {#if description}
                <p class="mt-2 text-sm font-light text-muted">{description}</p>
            {/if}
        </div>
        {#if actionLabel && onAction}
            <button
                onclick={onAction}
                class="rounded-xl bg-primary px-6 py-2.5 text-sm font-medium text-on-primary transition-opacity hover:opacity-90"
            >
                {actionLabel}
            </button>
        {/if}
    </div>
</div>
