import {getContext} from 'svelte';
import {rootKey, screenKey, scrollKey, stackKey} from './context';
import type {NavigationRoot, ScreenEntry, StackController} from './navigation.svelte';
import type {Navigation, ParamListBase, Route} from './types';

/** The navigation of the stack the calling screen belongs to. */
export function useNavigation<ParamList extends ParamListBase = ParamListBase>(): Navigation<ParamList> {
    const controller = getContext<StackController>(stackKey);
    const entry = getContext<ScreenEntry>(screenKey);
    return controller.navigation(entry) as Navigation<ParamList>;
}

/** The route of the calling screen; read it inside `$derived` to stay reactive. */
export function useRoute<ParamList extends ParamListBase = ParamListBase>(): Route<ParamList> {
    const entry = getContext<ScreenEntry>(screenKey);
    return {key: entry.key, name: entry.name, params: entry.params} as Route<ParamList>;
}

/** Focus is the top of the stack of the tab that is up. Wrap in `$derived` to stay reactive. */
export function useIsFocused(): boolean {
    return getContext<ScreenEntry>(screenKey).focused;
}

/** The element that scrolls the calling screen, for scroll position and scroll listeners. */
export function useScrollContainer(): () => HTMLElement | undefined {
    return getContext<() => HTMLElement | undefined>(scrollKey);
}

/**
 * Android's back key. Call it at the top of a screen's script to get the function
 * that takes the key while a mode is on:
 *
 *     const takeBack = useBackHandler();
 *     $effect(() => {
 *         if (!search.open) return;
 *         return takeBack(() => search.exit());
 *     });
 *
 * A mode owns one history entry, because the back key is a WebView history back:
 * on a root screen there would otherwise be no entry left to cancel it with.
 */
export function useBackHandler(): (listener: () => void) => () => void {
    const root = getContext<NavigationRoot>(rootKey);
    const entry = getContext<ScreenEntry>(screenKey);
    return (listener) => root.enterMode(entry, listener);
}
