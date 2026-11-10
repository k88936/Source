<script lang="ts">
    import {createBottomTabNavigator, NavigationContainer, type TabBarProps} from 'svelte-navigation';
    import type {TabParamList} from './params';
    import HomeTab from './tabs/HomeTab.svelte';
    import SettingsTab from './tabs/SettingsTab.svelte';
    import {homeScreens, settingsScreens} from './tabs/screens';

    const Tabs = createBottomTabNavigator<TabParamList>();

    const tabs = {
        Home: {component: HomeTab, screens: homeScreens},
        Settings: {component: SettingsTab, screens: settingsScreens},
    };
</script>

{#snippet tabBar({tabs, active, activate}: TabBarProps)}
    <nav class="flex border-t">
        {#each tabs as tab (tab)}
            <button
                class="flex-1 p-3"
                class:font-bold={active === tab}
                onclick={() => activate(tab)}
            >
                {tab}
            </button>
        {/each}
    </nav>
{/snippet}

<NavigationContainer>
    <Tabs.Navigator {tabs} {tabBar} />
</NavigationContainer>
