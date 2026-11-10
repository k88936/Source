import type {Component} from 'svelte';
import type {ParamListBase, StackNavigatorProps, StackScreenProps} from '../types';
import Navigator from './Navigator.svelte';
import Screen from './Screen.svelte';

/** Same shape as `@react-navigation/native-stack`: `<Stack.Navigator>` and `<Stack.Screen>`. */
export function createNativeStackNavigator<ParamList extends ParamListBase>() {
    return {
        Navigator: Navigator as unknown as Component<StackNavigatorProps<ParamList>>,
        Screen: Screen as unknown as Component<StackScreenProps<ParamList>>,
    };
}
