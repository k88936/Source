<script lang="ts">
    import {useNavigation, useRoute} from 'svelte-navigation';
    import type {HomeStackParamList} from '../params';

    const navigation = useNavigation<HomeStackParamList>();
    const route = $derived(useRoute<HomeStackParamList>());
</script>

<div class="space-y-3 p-4">
    <p id="detail-id" class="text-2xl font-semibold">Item {route.params?.id}</p>
    <p class="text-sm text-muted">Pushed on top of Home, which stays mounted below.</p>
    <button
        type="button"
        class="rounded-lg bg-gray px-4 py-2 text-sm"
        onclick={() => navigation.navigate('Detail', {id: String(Number(route.params?.id) + 1)})}
    >
        open next
    </button>
    <button type="button" class="rounded-lg bg-gray px-4 py-2 text-sm" onclick={() => navigation.goBack()}>
        go back
    </button>
</div>
