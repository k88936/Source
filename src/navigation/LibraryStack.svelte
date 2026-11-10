<script lang="ts">
    import {createNativeStackNavigator} from 'svelte-navigation';
    import {common} from '#lib/i18n/common.i18n';
    import {i18n} from '#lib/state/i18n.svelte';
    import Library from '#screens/library/Library.svelte';
    import LocalAlbum from '#screens/library/LocalAlbum.svelte';
    import LocalAlbums from '#screens/library/LocalAlbums.svelte';
    import Settings from '#screens/library/Settings.svelte';
    import Viewer from '#screens/viewer/Viewer.svelte';
    import OfflineBanner from './OfflineBanner.svelte';
    import type {LibraryStackParamList} from './params';

    const Stack = createNativeStackNavigator<LibraryStackParamList>();
    const c = $derived(i18n.pick(common));
</script>

{#snippet banner()}
    <OfflineBanner />
{/snippet}

<Stack.Navigator initialRouteName="Library" screenOptions={{headerBackLabel: c.back, headerBanner: banner}}>
    <Stack.Screen name="Library" component={Library} />
    <Stack.Screen name="LocalAlbums" component={LocalAlbums} />
    <Stack.Screen name="LocalAlbum" component={LocalAlbum} />
    <Stack.Screen name="Settings" component={Settings} />
    <Stack.Screen
        name="Viewer"
        component={Viewer}
        options={{presentation: 'modal', animation: 'slide_from_bottom', headerShown: false}}
    />
</Stack.Navigator>
