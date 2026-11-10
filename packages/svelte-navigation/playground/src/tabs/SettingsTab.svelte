<script lang="ts">
    import {createNativeStackNavigator} from 'svelte-navigation';
    import type {SettingsStackParamList} from '../params';
    import Settings from '../screens/Settings.svelte';

    const Stack = createNativeStackNavigator<SettingsStackParamList>();
</script>

<Stack.Navigator initialRouteName="Settings" screenOptions={{headerBackLabel: 'Back'}}>
    <Stack.Screen name="Settings" component={Settings} options={{title: 'Settings'}} />
</Stack.Navigator>
