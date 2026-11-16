<script lang="ts">
    import {useBackHandler, useNavigation} from 'svelte-navigation';
    import type {HomeStackParamList} from '../params';

    const navigation = useNavigation<HomeStackParamList>();

    let steps = $state(0);

    // A step is a mode: the back key takes one back before it closes the sheet.
    useBackHandler(
        () => steps !== 0,
        () => (steps -= 1),
    );
</script>

<div class="space-y-3 p-4">
    <p class="text-sm text-muted">A modal presentation: it covers the tab bar.</p>
    <p id="sheet-steps" class="text-2xl font-semibold">step {steps}</p>
    <button
        type="button"
        class="rounded-lg bg-gray px-4 py-2 text-sm"
        onclick={() => steps++}
    >
        take a step
    </button>
    <button type="button" class="rounded-lg bg-primary px-4 py-2 text-sm text-on-primary" onclick={() => navigation.goBack()}>
        close
    </button>
</div>
