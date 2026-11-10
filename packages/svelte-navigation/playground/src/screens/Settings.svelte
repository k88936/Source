<script lang="ts">
    let count = $state(0);
</script>

<div class="space-y-3 p-4">
    <p class="text-sm text-muted">A second tab with its own stack and its own state.</p>
    <p id="settings-count" class="text-2xl font-semibold">{count}</p>
    <button type="button" class="rounded-lg bg-gray px-4 py-2 text-sm" onclick={() => count++}>
        increment
    </button>
</div>
