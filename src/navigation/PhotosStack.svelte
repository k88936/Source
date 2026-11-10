<script lang="ts">
    import {createNativeStackNavigator, type StackHeaderProps} from 'svelte-navigation';
    import {common} from '#lib/i18n/common.i18n';
    import {i18n} from '#lib/state/i18n.svelte';
    import Photos from '#screens/photos/Photos.svelte';
    import Viewer from '#screens/viewer/Viewer.svelte';
    import AppHeader from './AppHeader.svelte';
    import OfflineBanner from './OfflineBanner.svelte';
    import type {PhotosStackParamList} from './params';

    const Stack = createNativeStackNavigator<PhotosStackParamList>();
    const c = $derived(i18n.pick(common));
</script>

{#snippet banner()}
    <OfflineBanner />
{/snippet}

{#snippet header(props: StackHeaderProps)}
    <AppHeader {...props} />
{/snippet}

<Stack.Navigator
    initialRouteName="Photos"
    screenOptions={{headerBackLabel: c.back, headerBanner: banner}}
    layerClass="bg-light text-dark"
    {header}
>
    <Stack.Screen name="Photos" component={Photos} />
    <Stack.Screen
        name="Viewer"
        component={Viewer}
        options={{presentation: 'modal', animation: 'slide_from_bottom', headerShown: false}}
    />
</Stack.Navigator>
