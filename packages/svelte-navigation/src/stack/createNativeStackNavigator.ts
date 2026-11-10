import type {Component} from 'svelte';
import type {ParamListBase, StackNavigatorProps} from '../types';
import Navigator from './Navigator.svelte';

/**
 * The screens live in the navigator's `screens` table, so a screen's component,
 * its options and its URL are declared once, next to each other.
 */
export function createNativeStackNavigator<ParamList extends ParamListBase>() {
    return {
        Navigator: Navigator as unknown as Component<StackNavigatorProps<ParamList>>,
    };
}
