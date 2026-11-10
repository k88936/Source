import {ThemePreference, themeManager} from '@immich/ui';
import {mount} from 'svelte';
import './app.css';
import App from './App.svelte';

// The inline script in `index.html` only paints light or dark before Svelte
// loads; once the app is up, the stored preference wins, including `system`.
if (localStorage.getItem('immich-ui-theme') === null) {
    themeManager.setPreference(ThemePreference.Dark);
}
themeManager.setPreference(themeManager.preference);

mount(App, {target: document.getElementById('app')!});
