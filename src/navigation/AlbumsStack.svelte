<script lang="ts">
    import {createNativeStackNavigator} from 'svelte-navigation';
    import {common} from '#lib/i18n/common.i18n';
    import {i18n} from '#lib/state/i18n.svelte';
    import Album from '#screens/albums/Album.svelte';
    import Albums from '#screens/albums/Albums.svelte';
    import Viewer from '#screens/viewer/Viewer.svelte';
    import OfflineBanner from './OfflineBanner.svelte';
    import type {AlbumsStackParamList} from './params';

    const Stack = createNativeStackNavigator<AlbumsStackParamList>();
    const c = $derived(i18n.pick(common));
</script>

{#snippet banner()}
    <OfflineBanner />
{/snippet}

<Stack.Navigator initialRouteName="Albums" screenOptions={{headerBackLabel: c.back, headerBanner: banner}}>
    <Stack.Screen name="Albums" component={Albums} />
    <Stack.Screen name="Album" component={Album} />
    <Stack.Screen
        name="Viewer"
        component={Viewer}
        options={{presentation: 'modal', animation: 'slide_from_bottom', headerShown: false}}
    />
</Stack.Navigator>
