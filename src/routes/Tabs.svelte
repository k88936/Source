<script lang="ts">
    import {createBottomTabNavigator, type TabBarProps} from 'svelte-navigation';
    import type {TabParamList} from './params';
    import {tabs} from './tabs';
    import TabBar from '#lib/components/shell/TabBar/TabBar.svelte';

    const Tabs = createBottomTabNavigator<TabParamList>();
</script>

{#snippet tabBar(props: TabBarProps)}
    <TabBar {...props} />
{/snippet}

<!-- Each tab owns its stack, so a detail page stays open behind a tab switch. -->
<Tabs.Navigator {tabs} {tabBar} />
