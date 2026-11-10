<script lang="ts">
    import {createNativeStackNavigator} from 'svelte-navigation';
    import {common} from '#lib/i18n/common.i18n';
    import {i18n} from '#lib/state/i18n.svelte';
    import Welcome from '#screens/welcome/Welcome.svelte';
    import type {WelcomeStackParamList} from './params';

    const Stack = createNativeStackNavigator<WelcomeStackParamList>();
    const c = $derived(i18n.pick(common));
</script>

<Stack.Navigator initialRouteName="Welcome" screenOptions={{headerBackLabel: c.back}}>
    <Stack.Screen name="Welcome" component={Welcome} options={{headerShown: false}} />
</Stack.Navigator>
