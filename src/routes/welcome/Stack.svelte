<script lang="ts">
    import {createNativeStackNavigator} from 'svelte-navigation';
    import {common} from '#lib/i18n/common.i18n';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import {welcomeScreens, type WelcomeStackParamList} from './route';

    const Stack = createNativeStackNavigator<WelcomeStackParamList>();
    const c = $derived(i18n.pick(common));
</script>

<Stack.Navigator
    screens={welcomeScreens}
    initialRouteName="Welcome"
    screenOptions={{headerBackLabel: c.back}}
    layerClass="bg-light text-dark"
/>
