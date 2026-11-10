<script lang="ts">
    import {createNativeStackNavigator} from 'svelte-navigation';
    import type {HomeStackParamList} from '../params';
    import {homeScreens} from './screens';

    const Stack = createNativeStackNavigator<HomeStackParamList>();
</script>

<Stack.Navigator screens={homeScreens} initialRouteName="Home" screenOptions={{headerBackLabel: 'Back'}} />
