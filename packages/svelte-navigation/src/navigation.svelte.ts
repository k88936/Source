import {untrack} from 'svelte';
import type {Component} from 'svelte';
import type {Navigation, ParamListBase, Route, StackScreen, StackOptions} from './types';

/** How long a popped layer stays around to play its exit transition. */
export const TRANSITION_MS = 280;

export type ScreenEntry = {
    key: string;
    name: string;
    params: Record<string, unknown>;
    /** Runtime options from `navigation.setOptions`, layered over the static ones. */
    options: Partial<StackOptions>;
    focused: boolean;
    /** Only a screen pushed while running slides in; the initial one is already there. */
    animate: boolean;
    /** True once this entry got its own history entry, so back can just go back. */
    historic: boolean;
    focus: Set<() => void>;
    blur: Set<() => void>;
    navigation?: Navigation;
};

/** One registered screen: the navigator's name for it plus its own declaration. */
export type StackScreenConfig = StackScreen & {name: string};

export type TabScreenConfig = {
    name: string;
    component: Component<Record<string, never>>;
};

/** The `history.state` of one stack screen, or of a mode that takes the back key. */
export type HistoryEntry = {
    tab: string;
    depth: number;
    name: string;
    params: Record<string, unknown>;
    /** A mode owns an entry of its own, so the back key can close it instead of the page. */
    mode?: boolean;
};

const base = import.meta.env.BASE_URL.replace(/\/$/, '');

/** Matches one path against a `'/albums/:id'` style pattern. */
export function matchPath(pattern: string, path: string): Record<string, string> | null {
    const expected = pattern.split('/').filter(Boolean);
    const actual = path.split('/').filter(Boolean);
    if (expected.length !== actual.length) return null;
    const params: Record<string, string> = {};
    for (const [index, part] of expected.entries()) {
        const value = actual[index];
        if (value === undefined) return null;
        if (part.startsWith(':')) params[part.slice(1)] = decodeURIComponent(value);
        else if (part !== value) return null;
    }
    return params;
}

export function buildPath(pattern: string, params: Record<string, unknown>): string {
    return pattern.replace(/:([A-Za-z0-9_]+)/g, (_, name: string) => encodeURIComponent(String(params[name])));
}

function resolveOptions<T>(
    value: T | ((route: Route<ParamListBase>) => T) | undefined,
    route: Route<ParamListBase>,
): T {
    return typeof value === 'function' ? (value as (route: Route<ParamListBase>) => T)(route) : (value as T);
}

let counter = 0;

/**
 * The navigation tree of one app: which tab is up, and the history entries that
 * mirror the stack it shows. Every stack screen owns one `history` entry, so the
 * Android back key, which is a WebView history back, pops the stack for free.
 */
export class NavigationRoot {
    active = $state('');
    tabs = $state<string[]>([]);
    controllers = new Map<string, StackController>();
    /** Tab name -> screen name -> URL pattern, collected from the screens themselves. */
    private patterns = new Map<string, Record<string, string>>();
    private pending: {tab: string; name: string; params: Record<string, unknown>} | null = null;
    private current: HistoryEntry | null = null;
    private currentUrl = '';
    // The header reads `hasMode` while it renders, so the modes have to be state: a
    // plain array leaves the back button hidden after the mode is gone. They are
    // untracked everywhere else, because the effect that opens a mode must not
    // depend on the modes it writes.
    private modes = $state.raw<{entryKey: string; listener: () => void}[]>([]);
    private started = false;

    /**
     * The paths a level answers on. A tab declares them before it is mounted, so a
     * cold start can pick the tab from the URL without a second linking table.
     */
    registerPatterns(tab: string, screens: Record<string, StackScreen>): void {
        const paths: Record<string, string> = {};
        for (const [name, screen] of Object.entries(screens)) {
            if (screen.path) paths[name] = screen.path;
        }
        this.patterns.set(tab, paths);
        if (!this.started) this.resolve();
    }

    unregisterPatterns(tab: string): void {
        this.patterns.delete(tab);
    }

    /** Matches the opening URL to the deepest declared path, until the tree is up. */
    private resolve(): void {
        const path = location.pathname.slice(base.length) || '/';
        let weight = 0;
        this.pending = null;
        for (const [tab, screens] of this.patterns) {
            for (const [name, pattern] of Object.entries(screens)) {
                const params = matchPath(pattern, path);
                if (!params) continue;
                const depth = pattern.split('/').filter(Boolean).length;
                if (depth < weight) continue;
                weight = depth;
                this.pending = {tab, name, params};
            }
        }
        this.active = this.pending?.tab ?? '';
    }

    /** Called by the tab navigator once the tab order is known. */
    setTabs(names: string[]): void {
        this.tabs = names;
        if (this.active) return;
        this.active = this.pending?.tab ?? names[0] ?? '';
    }

    takePending(tab: string): {name: string; params: Record<string, unknown>} | undefined {
        if (this.pending?.tab !== tab) return undefined;
        const taken = this.pending;
        this.pending = null;
        return taken;
    }

    attach(controller: StackController): void {
        this.started = true;
        this.controllers.set(controller.tab, controller);
        if (!this.active) this.active = controller.tab;
        if (this.active === controller.tab) this.replace(controller);
        this.syncFocus();
    }

    detach(controller: StackController): void {
        this.controllers.delete(controller.tab);
        if (this.controllers.size > 0) return;
        this.tabs = [];
        this.active = '';
    }

    /** A tab press: `popToTop` on the tab already up, a switch otherwise. */
    activate(tab: string): void {
        if (this.active === tab) {
            this.controllers.get(tab)?.popToTop();
            return;
        }
        this.active = tab;
        const controller = this.controllers.get(tab);
        if (controller) this.replace(controller);
        this.syncFocus();
    }

    navigate(from: StackController, name: string, params: Record<string, unknown>): void {
        const controller = this.controllers.get(from.tab);
        if (controller?.configs[name]) {
            controller.push(name, params);
            return;
        }
        if (this.tabs.includes(name)) this.activate(name);
    }

    back(controller: StackController): void {
        const top = controller.entries.at(-1);
        if (!top || controller.entries.length <= 1) return;
        if (top.historic) history.back();
        else controller.pop();
    }

    replace(controller: StackController): void {
        const top = controller.entries.at(-1);
        if (!top) return;
        this.write('replace', controller, top, this.url(controller.tab, top));
    }

    push(controller: StackController): void {
        const top = controller.entries.at(-1);
        if (!top) return;
        top.historic = true;
        this.write('push', controller, top, this.url(controller.tab, top));
    }

    /**
     * A mode of a screen that takes the back key, e.g. search or a selection.
     * It owns a history entry of its own: without one, the back key on a root
     * screen would leave the WebView instead of reaching the app at all.
     */
    enterMode(entry: ScreenEntry, listener: () => void): () => void {
        const mode = {entryKey: entry.key, listener};
        this.modes = [...untrack(() => this.modes), mode];
        const state = {...this.current, mode: true} as HistoryEntry;
        this.current = state;
        history.pushState(state, '', this.currentUrl);
        return () => this.leaveMode(mode);
    }

    /** True while the screen takes the back key, so the header can drop its own back button. */
    hasMode(entry: ScreenEntry): boolean {
        return this.modes.some((mode) => mode.entryKey === entry.key);
    }

    handlePop = (event: PopStateEvent): void => {
        const state = event.state as HistoryEntry | null;
        if (!state) return;
        // The event carries the entry being entered, so the one being left is the current one.
        const leaving = this.current;
        this.current = state;
        this.currentUrl = location.pathname + location.search;

        if (leaving?.mode) {
            // Back out of a mode: it closes, and neither the stack nor the URL moves.
            const mode = this.modes.at(-1);
            if (mode) {
                this.modes = this.modes.slice(0, -1);
                mode.listener();
                return;
            }
        }

        if (state.mode) {
            // Back onto a mode entry that has a screen above it: pop that screen and
            // put the mode entry back on top, so the next press closes the mode.
            const controller = this.controllers.get(state.tab);
            if (controller && controller.entries.length > state.depth) {
                controller.restoreTo(state);
                history.pushState(state, '', this.currentUrl);
                this.syncFocus();
                return;
            }
        }

        this.active = state.tab;
        this.controllers.get(state.tab)?.restoreTo(state);
        this.syncFocus();
    };

    start(): void {
        addEventListener('popstate', this.handlePop);
    }

    stop(): void {
        removeEventListener('popstate', this.handlePop);
    }

    syncFocus(): void {
        for (const [tab, controller] of this.controllers) {
            const top = this.active === tab ? controller.entries.at(-1) : undefined;
            controller.setFocus(top?.key ?? null);
        }
    }

    /** A mode that closes on its own takes its entry with it, unless it was the one popped. */
    private leaveMode(mode: {entryKey: string; listener: () => void}): void {
        const modes = untrack(() => this.modes);
        if (!modes.includes(mode)) return;
        this.modes = modes.filter((current) => current !== mode);
        if (!(history.state as HistoryEntry | null)?.mode) return;
        history.back();
    }

    private url(tab: string, entry: ScreenEntry): string {
        const pattern = this.patterns.get(tab)?.[entry.name];
        if (pattern === undefined) return this.currentUrl || `${base}/`;
        return base + buildPath(pattern, entry.params);
    }

    private write(mode: 'push' | 'replace', controller: StackController, entry: ScreenEntry, url: string): void {
        const state: HistoryEntry = {
            tab: controller.tab,
            depth: controller.entries.length,
            name: entry.name,
            // `history.state` is structured-cloned, so the reactive params cannot go in as they are.
            params: $state.snapshot(entry.params) as Record<string, unknown>,
        };
        this.current = state;
        this.currentUrl = url;
        if (mode === 'push') {
            history.pushState(state, '', url);
            return;
        }
        history.replaceState(state, '', url);
        // The replaced entry no longer exists, so the modes that pointed at it are over.
        this.dropOtherModes(entry.key);
    }

    private dropOtherModes(keepKey: string): void {
        const modes = untrack(() => this.modes);
        const stale = modes.filter((mode) => mode.entryKey !== keepKey);
        if (stale.length === 0) return;
        this.modes = modes.filter((mode) => mode.entryKey === keepKey);
        for (const mode of stale) mode.listener();
    }
}

/** The screen stack of one tab, and the navigation handed to its screens. */
export class StackController {
    readonly root: NavigationRoot;
    readonly tab: string;
    readonly initialRouteName: string;
    entries = $state<ScreenEntry[]>([]);
    leaving = $state<ScreenEntry[]>([]);
    configs = $state<Record<string, StackScreenConfig>>({});
    private screenOptions: () => StackOptions | ((route: Route<ParamListBase>) => StackOptions);

    constructor(options: {
        root: NavigationRoot;
        tab: string;
        initialRouteName: string;
        screenOptions: () => StackOptions | ((route: Route<ParamListBase>) => StackOptions);
    }) {
        this.root = options.root;
        this.tab = options.tab;
        this.initialRouteName = options.initialRouteName;
        this.screenOptions = options.screenOptions;
    }

    /**
     * The navigator hands over its whole table at once, so a screen cannot be
     * registered in one place and addressed by name in another.
     */
    setScreens(screens: Record<string, StackScreen>): void {
        const configs: Record<string, StackScreenConfig> = {};
        for (const [name, screen] of Object.entries(screens)) configs[name] = {...screen, name};
        this.configs = configs;
        // A tab's paths belong to the tab navigator, which knows about a tab before its
        // stack is mounted. Only the root stack has no such owner, so it reports itself.
        if (!this.tab) this.root.registerPatterns('', screens);
    }

    mount(): void {
        const pending = this.root.takePending(this.tab);
        const initial = this.build(this.initialRouteName, {});
        this.entries =
            pending && pending.name !== this.initialRouteName
                ? [initial, this.build(pending.name, pending.params)]
                : [initial];
        this.root.attach(this);
    }

    unmount(): void {
        if (!this.tab) this.root.unregisterPatterns('');
        this.root.detach(this);
    }

    optionsFor(entry: ScreenEntry): StackOptions {
        const route: Route<ParamListBase> = {key: entry.key, name: entry.name, params: entry.params};
        const base = resolveOptions(this.screenOptions(), route) ?? {};
        const own = resolveOptions(this.configs[entry.name]?.options, route) ?? {};
        return {...base, ...own, ...entry.options};
    }

    navigation(entry: ScreenEntry): Navigation {
        if (entry.navigation) return entry.navigation;
        const navigation: Navigation = {
            navigate: (name: string, params?: Record<string, unknown>) =>
                this.root.navigate(this, name, params ?? {}),
            goBack: () => this.root.back(this),
            canGoBack: () => this.entries.length > 1,
            popToTop: () => this.popToTop(),
            reset: (name: string, params?: Record<string, unknown>) => this.reset(name, params ?? {}),
            setOptions: (options: Partial<StackOptions>) => {
                // Reading the current options here would make an effect that calls
                // `setOptions` depend on what it writes, so the read is untracked.
                const current = untrack(() => entry.options);
                const changed = Object.entries(options).some(
                    ([key, value]) => current[key as keyof StackOptions] !== value,
                );
                if (!changed) return;
                entry.options = {...current, ...options};
            },
            addListener: (type: 'focus' | 'blur', listener: () => void) => {
                entry[type].add(listener);
                return () => {
                    entry[type].delete(listener);
                };
            },
        };
        entry.navigation = navigation;
        return navigation;
    }

    push(name: string, params: Record<string, unknown> = {}): void {
        this.entries = [...this.entries, this.build(name, params, true)];
        this.root.push(this);
        this.root.syncFocus();
    }

    pop(): void {
        if (this.entries.length <= 1) return;
        this.drop([this.entries.at(-1) as ScreenEntry]);
        this.root.replace(this);
        this.root.syncFocus();
    }

    popToTop(): void {
        if (this.entries.length <= 1) return;
        const top = this.entries.at(-1) as ScreenEntry;
        this.entries = this.entries.slice(0, 1);
        this.keepExiting([top]);
        this.root.replace(this);
        this.root.syncFocus();
    }

    reset(name: string, params: Record<string, unknown> = {}): void {
        this.entries = [this.build(name, params)];
        this.root.replace(this);
        this.root.syncFocus();
    }

    restoreTo(state: HistoryEntry): void {
        if (state.depth === this.entries.length) return;
        if (state.depth < this.entries.length) {
            this.drop(this.entries.slice(state.depth));
        } else if (state.depth === this.entries.length + 1) {
            this.entries = [...this.entries, this.build(state.name, state.params, true)];
        }
        this.root.syncFocus();
    }

    setFocus(key: string | null): void {
        for (const entry of this.entries) {
            const focused = entry.key === key;
            if (entry.focused === focused) continue;
            entry.focused = focused;
            for (const listener of [...(focused ? entry.focus : entry.blur)]) listener();
        }
    }

    private drop(removed: ScreenEntry[]): void {
        const keys = new Set(removed.map((entry) => entry.key));
        this.entries = this.entries.filter((entry) => !keys.has(entry.key));
        this.keepExiting(removed);
    }

    private keepExiting(removed: ScreenEntry[]): void {
        const keys = new Set(removed.map((entry) => entry.key));
        this.leaving = [...this.leaving, ...removed];
        setTimeout(() => {
            this.leaving = this.leaving.filter((entry) => !keys.has(entry.key));
        }, TRANSITION_MS);
    }

    private build(name: string, params: Record<string, unknown>, animate = false): ScreenEntry {
        return {
            key: `screen-${++counter}`,
            name,
            params,
            options: {},
            focused: false,
            animate,
            historic: false,
            focus: new Set(),
            blur: new Set(),
        };
    }
}
