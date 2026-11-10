<script lang="ts">
    import {createNativeStackNavigator, type StackHeaderProps} from 'svelte-navigation';
    import AppHeader from '#lib/components/shell/AppHeader/AppHeader.svelte';
    import OfflineBanner from '#lib/components/shell/OfflineBanner/OfflineBanner.svelte';
    import {common} from '#lib/i18n/common.i18n';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import {syncScreens, type SyncStackParamList} from './route';

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
    screens={syncScreens}
    initialRouteName="Sync"
    screenOptions={{headerBackLabel: c.back, headerBanner: banner}}
    layerClass="bg-light text-dark"
    {header}
/>
