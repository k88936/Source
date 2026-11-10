import type {StackOptions} from 'svelte-navigation';
import ViewerModal from './ViewerModal.svelte';

/**
 * The viewer is one modal shared by every stack: what it shows is injected when
 * it is opened, so each stack declares the same screen and nothing else.
 */
export const viewerScreen: {component: typeof ViewerModal; options: StackOptions} = {
    component: ViewerModal,
    options: {presentation: 'modal', animation: 'slide_from_bottom', headerShown: false},
};
