<script lang="ts">
    import {ConfirmModal} from '@immich/ui';
    import {mdiAlertCircleOutline} from '@mdi/js';

    let {
        open,
        title,
        message,
        confirmLabel,
        danger = false,
        onClose,
        onConfirm,
    }: {
        open: boolean;
        title: string;
        message: string;
        confirmLabel: string;
        /** Destructive actions tint the confirm button with the danger token. */
        danger?: boolean;
        onClose: () => void;
        onConfirm: () => void;
    } = $props();
</script>

<!-- WebKitGTK's own `confirm()` is unthemeable and blocks the webview, so actions
     route through the library's confirm dialog instead. -->
{#if open}
    <ConfirmModal
        {title}
        prompt={message}
        confirmText={confirmLabel}
        confirmColor={danger ? 'danger' : 'primary'}
        icon={danger ? mdiAlertCircleOutline : true}
        onClose={(confirmed) => (confirmed ? onConfirm() : onClose())}
    />
{/if}
