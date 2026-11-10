<script lang="ts">
    import {useNavigation} from 'svelte-navigation';
    import type {HomeStackParamList} from '../params';

    const navigation = useNavigation<HomeStackParamList>();

    let count = $state(0);
    let text = $state('');

    const rows = Array.from({length: 60}, (_, index) => index + 1);
</script>

<div class="space-y-3 p-4">
    <p class="text-sm text-muted">This counter and the text must survive a push and a back.</p>
    <p id="count" class="text-2xl font-semibold">{count}</p>
    <input class="w-full rounded-lg bg-gray px-4 py-2.5 text-sm outline-none" placeholder="type something" bind:value={text} />
    <button
        type="button"
        class="rounded-lg bg-primary px-4 py-2 text-sm text-on-primary"
        onclick={() => count++}
    >
        increment
    </button>
    <button
        type="button"
        class="rounded-lg bg-gray px-4 py-2 text-sm"
        onclick={() => navigation.navigate('Detail', {id: '42'})}
    >
        open detail
    </button>
    <button
        type="button"
        class="rounded-lg bg-gray px-4 py-2 text-sm"
        onclick={() => navigation.navigate('Sheet')}
    >
        open sheet
    </button>
    {#each rows as row (row)}
        <p class="border-b border-line py-3 text-sm">row {row}</p>
    {/each}
</div>
