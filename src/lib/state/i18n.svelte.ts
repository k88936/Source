import type {Bundle, Locale} from '#lib/i18n/bundle';

const KEY = 'source-locale';

const TAGS: Record<Locale, string> = {zh: 'zh-CN', en: 'en-US'};

function initial(): Locale {
    const stored = localStorage.getItem(KEY);
    if (stored === 'zh' || stored === 'en') return stored;
    return navigator.language.toLowerCase().startsWith('zh') ? 'zh' : 'en';
}

class I18n {
    locale = $state<Locale>(initial());

    /** BCP 47 tag for `Intl` and `<html lang>`. */
    get tag(): string {
        return TAGS[this.locale];
    }

    /** Read only inside `$derived`, so the text follows the locale. */
    pick<T>(text: Bundle<T>): T {
        return text[this.locale];
    }

    set(next: Locale): void {
        this.locale = next;
        localStorage.setItem(KEY, next);
        document.documentElement.lang = TAGS[next];
    }
}

export const i18n = new I18n();

document.documentElement.lang = i18n.tag;
