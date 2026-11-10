<script lang="ts">
    import {createNativeStackNavigator} from 'svelte-navigation';
    import {common} from '#lib/i18n/common.i18n';
    import {i18n} from '#lib/state/i18n.svelte';
    import Sync from '#screens/sync/Sync.svelte';
    import OfflineBanner from './OfflineBanner.svelte';
    import type {SyncStackParamList} from './params';

    const Stack = createNativeStackNavigator<SyncStackParamList>();
    const c = $derived(i18n.pick(common));
</script>

{#snippet banner()}
    <OfflineBanner />
{/snippet}

<Stack.Navigator initialRouteName="Sync" screenOptions={{headerBackLabel: c.back, headerBanner: banner}}>
    <Stack.Screen name="Sync" component={Sync} />
</Stack.Navigator>
