import type {Component, Snippet} from 'svelte';

/** Every navigator declares its screens and their params here. */
export type ParamListBase = Record<string, Record<string, unknown> | undefined>;

export type Route<ParamList extends ParamListBase = ParamListBase, Name extends keyof ParamList = keyof ParamList> = {
    key: string;
    name: Name;
    params: ParamList[Name];
};

/** Same names as `@react-navigation/native-stack`. */
export type Animation = 'slide_from_right' | 'fade' | 'slide_from_bottom' | 'none';

export type HeaderOptions = {
    title?: string;
    subtitle?: string;
    headerShown?: boolean;
    headerTransparent?: boolean;
    /** Aria label of the header's back button. */
    headerBackLabel?: string;
    /** Rendered above the bar, e.g. the offline strip. */
    headerBanner?: Snippet;
    /** Replaces the title and the subtitle, e.g. by a search field. */
    headerContent?: Snippet;
    headerActions?: Snippet;
};

export type StackOptions = HeaderOptions & {
    /** `modal` slides up over the tab bar and keeps the URL of the screen below. */
    presentation?: 'card' | 'modal';
    animation?: Animation;
};

/** What a header needs from the navigator: the navigator decides, the app paints. */
export type StackHeaderProps = {
    options: StackOptions;
    /** False while the stack has a single screen or a mode owns the back key. */
    canBack: boolean;
    back: () => void;
};

/** What a tab bar needs from the navigator. */
export type TabBarProps = {
    tabs: string[];
    active: string;
    activate: (name: string) => void;
};

/**
 * One screen of a stack. Everything the navigator needs about it lives here, so
 * a screen cannot be declared in one place and given a URL in another.
 */
export type StackScreen<ParamList extends ParamListBase = ParamListBase> = {
    component: Component<Record<string, never>>;
    /** URL pattern, e.g. `/albums/:id`. A screen without one keeps the URL below it. */
    path?: string;
    options?: StackOptions | ((route: Route<ParamList>) => StackOptions);
};

/** Name -> screen, checked against the stack's param list so neither can be forgotten. */
export type StackScreens<ParamList extends ParamListBase> = {
    [Name in keyof ParamList & string]: StackScreen<ParamList>;
};

export type StackNavigatorProps<ParamList extends ParamListBase> = {
    screens: StackScreens<ParamList>;
    initialRouteName: keyof ParamList & string;
    screenOptions?: StackOptions | ((route: Route<ParamList>) => StackOptions);
    /** Painted per layer, above the scroll container and below the status bar. */
    header?: Snippet<[StackHeaderProps]>;
    /** Classes for the layer itself, so the app owns its surface colour. */
    layerClass?: string;
};

/**
 * One tab: the stack it shows. The navigator owns the table, so the tab list and
 * the paths of a tab are known before any of its stacks is mounted.
 */
export type TabScreen = {
    component: Component<Record<string, never>>;
    screens: Record<string, StackScreen>;
};

/** Name -> tab, checked against the tab param list so none can be forgotten. */
export type TabScreens<ParamList extends ParamListBase> = {
    [Name in keyof ParamList & string]: TabScreen;
};

export type TabNavigatorProps<ParamList extends ParamListBase> = {
    tabs: TabScreens<ParamList>;
    /** Painted below the tab content. */
    tabBar: Snippet<[TabBarProps]>;
};

export type Navigation<ParamList extends ParamListBase = ParamListBase> = {
    navigate<Name extends keyof ParamList>(name: Name, params?: ParamList[Name]): void;
    goBack(): void;
    canGoBack(): boolean;
    popToTop(): void;
    reset<Name extends keyof ParamList>(name: Name, params?: ParamList[Name]): void;
    setOptions(options: Partial<StackOptions>): void;
    addListener(type: 'focus' | 'blur', listener: () => void): () => void;
};
