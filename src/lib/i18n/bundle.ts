export type Locale = 'zh' | 'en';

/** Both translations of one module's words; `en` is the type source. */
export type Bundle<T> = {en: T; zh: T};

export function bundle<T>(en: T, zh: NoInfer<T>): Bundle<T> {
    return {en, zh};
}
