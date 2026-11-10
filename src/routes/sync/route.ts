import type {StackScreens} from 'svelte-navigation';
import type {TabParamList} from '../params';
import Sync from './Sync.svelte';

export type SyncStackParamList = {
    Sync: undefined;
};

export type SyncNavParamList = SyncStackParamList & TabParamList;

export const syncScreens = {
    Sync: {component: Sync, path: '/sync'},
} satisfies StackScreens<SyncStackParamList>;
