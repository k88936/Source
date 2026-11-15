import {ThemePreference, themeManager, toastManager} from '@immich/ui';
import {mount} from 'svelte';
import './app.css';
import App from './App.svelte';

// The inline script in `index.html` only paints light or dark before Svelte
// loads; once the app is up, the stored preference wins, including `system`.
if (localStorage.getItem('immich-ui-theme') === null) {
    themeManager.setPreference(ThemePreference.Dark);
}
themeManager.setPreference(themeManager.preference);

// The toast panel is mounted on `<body>`, outside the shell, so it gets the
// status-bar inset through its own class instead of the shell's padding.
toastManager.setOptions({class: 'toast-safe-top'});

mount(App, {target: document.getElementById('app')!});
