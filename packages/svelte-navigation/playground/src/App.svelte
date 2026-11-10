<script lang="ts">
    import {createBottomTabNavigator, NavigationContainer} from 'svelte-navigation';
    import type {TabParamList} from './params';
    import HomeTab from './tabs/HomeTab.svelte';
    import SettingsTab from './tabs/SettingsTab.svelte';
    import Square from './Square.svelte';
    import Circle from './Circle.svelte';

    const Tabs = createBottomTabNavigator<TabParamList>();

    const linking = {
        Home: {Home: '/home', Detail: '/home/:id'},
        Settings: {Settings: '/settings'},
    };
</script>

<NavigationContainer {linking}>
    <Tabs.Navigator>
        <Tabs.Screen name="Home" component={HomeTab} options={{title: 'Home', icon: Square}} />
        <Tabs.Screen name="Settings" component={SettingsTab} options={{title: 'Settings', icon: Circle}} />
    </Tabs.Navigator>
</NavigationContainer>
