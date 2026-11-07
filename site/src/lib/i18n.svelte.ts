export type Locale = 'zh' | 'en';

const KEY = 'source-locale';

export const TAGS: Record<Locale, string> = {zh: 'zh-CN', en: 'en-US'};

function initial(): Locale {
    const stored = localStorage.getItem(KEY);
    if (stored === 'zh' || stored === 'en') return stored;
    return navigator.language.toLowerCase().startsWith('zh') ? 'zh' : 'en';
}

/**
 * The site shares the app's `source-locale` key: the same-origin iframe reads it
 * too, so switching here switches the app as well.
 */
class SiteI18n {
    locale = $state<Locale>(initial());

    get tag(): string {
        return TAGS[this.locale];
    }

    get other(): Locale {
        return this.locale === 'zh' ? 'en' : 'zh';
    }

    pick<T>(text: Record<Locale, T>): T {
        return text[this.locale];
    }

    set(next: Locale): void {
        this.locale = next;
        localStorage.setItem(KEY, next);
        document.documentElement.lang = TAGS[next];
    }

    /** Follow the app's own switch, without writing the key back to it. */
    adopt(next: Locale): void {
        this.locale = next;
        document.documentElement.lang = TAGS[next];
    }
}

export const i18n = new SiteI18n();

document.documentElement.lang = i18n.tag;

/** The same-origin iframe is another document, so its switch arrives here. */
window.addEventListener('storage', (event) => {
    if (event.key !== KEY) return;
    if (event.newValue === 'zh' || event.newValue === 'en') i18n.adopt(event.newValue);
});
