<script lang="ts">
    import {createNativeStackNavigator, type StackHeaderProps} from 'svelte-navigation';
    import {common} from '#lib/i18n/common.i18n';
    import {i18n} from '#lib/state/i18n.svelte';
    import Sync from '#screens/sync/Sync.svelte';
    import AppHeader from './AppHeader.svelte';
    import OfflineBanner from './OfflineBanner.svelte';
    import type {SyncStackParamList} from './params';

    const Stack = createNativeStackNavigator<SyncStackParamList>();
    const c = $derived(i18n.pick(common));
</script>

{#snippet banner()}
    <OfflineBanner />
{/snippet}

{#snippet header(props: StackHeaderProps)}
    <AppHeader {...props} />
{/snippet}

<Stack.Navigator
    initialRouteName="Sync"
    screenOptions={{headerBackLabel: c.back, headerBanner: banner}}
    layerClass="bg-light text-dark"
    {header}
>
    <Stack.Screen name="Sync" component={Sync} />
</Stack.Navigator>
