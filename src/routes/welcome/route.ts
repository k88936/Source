import type {StackScreens} from 'svelte-navigation';
import Welcome from './Welcome.svelte';

export type WelcomeStackParamList = {
    Welcome: undefined;
};

export const welcomeScreens = {
    Welcome: {component: Welcome, path: '/welcome', options: {headerShown: false}},
} satisfies StackScreens<WelcomeStackParamList>;
