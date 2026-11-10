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

export type TabOptions = {
    title?: string;
    icon?: Component<{size?: number}>;
};

export type StackNavigatorProps<ParamList extends ParamListBase> = {
    initialRouteName: keyof ParamList & string;
    screenOptions?: StackOptions | ((route: Route<ParamList>) => StackOptions);
    children: Snippet;
};

export type StackScreenProps<ParamList extends ParamListBase> = {
    name: keyof ParamList & string;
    component: Component<Record<string, never>>;
    options?: StackOptions | ((route: Route<ParamList>) => StackOptions);
};

export type TabNavigatorProps<ParamList extends ParamListBase> = {
    screenOptions?: TabOptions | ((route: Route<ParamList>) => TabOptions);
    /** Aria label of the bar, in the app's language. */
    ariaLabel?: string;
    children: Snippet;
};

export type TabScreenProps<ParamList extends ParamListBase> = {
    name: keyof ParamList & string;
    component: Component<Record<string, never>>;
    options?: TabOptions | ((route: Route<ParamList>) => TabOptions);
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
