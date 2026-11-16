import {ThemePreference, themeManager, toastManager} from '@immich/ui';
import {mount} from 'svelte';
import './app.css';
import App from './App.svelte';

const THEME_KEY = 'immich-ui-theme';

// The landing page frames this app, so its light/dark switch arrives as a write
// to the shared key instead of a click inside this document.
window.addEventListener('storage', (event) => {
    if (event.key !== THEME_KEY || event.newValue === null) return;
    const preference = JSON.parse(event.newValue);
    if (preference === ThemePreference.Light || preference === ThemePreference.Dark) {
        themeManager.setPreference(preference);
    }
});

// The inline script in `index.html` only paints light or dark before Svelte
// loads; once the app is up, the stored preference wins, including `system`.
if (localStorage.getItem(THEME_KEY) === null) {
    themeManager.setPreference(ThemePreference.Dark);
}
themeManager.setPreference(themeManager.preference);

// The toast panel is mounted on `<body>`, outside the shell, so it gets the
// status-bar inset through its own class instead of the shell's padding.
toastManager.setOptions({class: 'toast-safe-top'});

mount(App, {target: document.getElementById('app')!});
