export type Theme = 'dark' | 'light';

/** The key the app reads at boot, so the site's switch drives the framed app too. */
const KEY = 'immich-ui-theme';

function parse(raw: string | null): Theme | null {
    if (raw === 'dark' || raw === '"dark"') return 'dark';
    if (raw === 'light' || raw === '"light"') return 'light';
    return null;
}

class SiteTheme {
    value = $state<Theme>(
        parse(localStorage.getItem(KEY)) ?? (matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'),
    );

    set(next: Theme): void {
        this.value = next;
        localStorage.setItem(KEY, JSON.stringify(next));
        document.documentElement.dataset.theme = next;
    }

    /** Follow the app's own switch, without writing the key back to it. */
    adopt(next: Theme): void {
        this.value = next;
        document.documentElement.dataset.theme = next;
    }
}

export const theme = new SiteTheme();

document.documentElement.dataset.theme = theme.value;

/** The same-origin iframe is another document, so its switch arrives here. */
window.addEventListener('storage', (event) => {
    if (event.key !== KEY) return;
    const next = parse(event.newValue);
    if (next) theme.adopt(next);
});
