<script lang="ts">
    import {createNativeStackNavigator} from 'svelte-navigation';
    import type {HomeStackParamList} from '../params';
    import Home from '../screens/Home.svelte';
    import Detail from '../screens/Detail.svelte';
    import Sheet from '../screens/Sheet.svelte';

    const Stack = createNativeStackNavigator<HomeStackParamList>();
</script>

<Stack.Navigator initialRouteName="Home" screenOptions={{headerBackLabel: 'Back'}}>
    <Stack.Screen name="Home" component={Home} options={{title: 'Home'}} />
    <Stack.Screen name="Detail" component={Detail} options={({params}) => ({title: `Item ${params.id}`})} />
    <Stack.Screen
        name="Sheet"
        component={Sheet}
        options={{presentation: 'modal', animation: 'slide_from_bottom', title: 'Sheet'}}
    />
</Stack.Navigator>
