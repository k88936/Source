import type {Component} from 'svelte';
import type {ParamListBase, TabNavigatorProps, TabScreenProps} from '../types';
import Navigator from './Navigator.svelte';
import Screen from './Screen.svelte';

/** Same shape as `@react-navigation/bottom-tabs`: `<Tabs.Navigator>` and `<Tabs.Screen>`. */
export function createBottomTabNavigator<ParamList extends ParamListBase>() {
    return {
        Navigator: Navigator as unknown as Component<TabNavigatorProps<ParamList>>,
        Screen: Screen as unknown as Component<TabScreenProps<ParamList>>,
    };
}
