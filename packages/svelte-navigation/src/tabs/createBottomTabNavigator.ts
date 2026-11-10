import type {Component} from 'svelte';
import type {ParamListBase, TabNavigatorProps} from '../types';
import Navigator from './Navigator.svelte';

/**
 * Same shape as `@react-navigation/bottom-tabs` static API: the navigator owns the
 * tabs, and each tab carries the stack it shows.
 */
export function createBottomTabNavigator<ParamList extends ParamListBase>() {
    return {Navigator: Navigator as unknown as Component<TabNavigatorProps<ParamList>>};
}
