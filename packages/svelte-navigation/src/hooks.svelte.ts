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
 * Android's back key. Call it at the top of a screen's script: while the condition
 * is true the key is taken and `onBack` closes what the condition stands for.
 *
 *     useBackHandler(() => search.open, () => search.exit());
 *
 * The effect that takes the key is this hook's own, so a caller cannot release it
 * late or leave it behind: the condition falling false or the screen going away
 * is all it takes to give the key back.
 *
 * A mode owns one history entry, because the back key is a WebView history back:
 * on a root screen there would otherwise be no entry left to cancel it with.
 */
export function useBackHandler(visible: () => boolean, onBack: () => void): void {
    takeBack(visible, onBack, false);
}

/**
 * Android's back key for a dialog opened over a screen, which takes the key the
 * same way a mode does. It leaves the screen's header in place, so the header
 * keeps its own back arrow while the dialog covers it.
 */
export function useOverlayBackHandler(visible: () => boolean, onBack: () => void): void {
    takeBack(visible, onBack, true);
}

function takeBack(visible: () => boolean, onBack: () => void, overlay: boolean): void {
    const root = getContext<NavigationRoot>(rootKey);
    const entry = getContext<ScreenEntry>(screenKey);
    // The condition is derived, so the effect restarts only when it flips: an input
    // that moves under a condition that stays true must not drop the mode it holds.
    const taken = $derived(visible());
    $effect(() => {
        if (!taken) return;
        return root.enterMode(entry, onBack, overlay);
    });
}
