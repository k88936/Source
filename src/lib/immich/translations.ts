import {setTranslations} from '@immich/ui';
import type {Locale} from '#lib/i18n/bundle';
import {common} from '#lib/i18n/common.i18n';
import {immich} from '#lib/i18n/immich.i18n';

/**
 * Hand the app's own words to `@immich/ui`, whose components render a few labels
 * (modal defaults, password toggle, toast titles) without a prop to override them.
 */
export function syncTranslations(locale: Locale): void {
    const words = common[locale];
    const labels = immich[locale];
    setTranslations({
        cancel: words.cancel,
        close: words.close,
        confirm: words.confirm,
        save: words.save,
        hide_password: labels.hidePassword,
        open_calendar: labels.openCalendar,
        open_menu: labels.openMenu,
        prompt_default: labels.promptDefault,
        search_no_results: labels.searchNoResults,
        search_placeholder: labels.searchPlaceholder,
        show_password: labels.showPassword,
        toast_danger_title: labels.toastDanger,
        toast_info_title: labels.toastInfo,
        toast_success_title: labels.toastSuccess,
        toast_warning_title: labels.toastWarning,
        dark_theme: labels.toggleDarkTheme,
    });
}
