import {fileURLToPath} from 'node:url';
import {svelte} from '@sveltejs/vite-plugin-svelte';
import {defineConfig} from 'vite';

/**
 * The landing page: one centered phone that frames the demo build from
 * `/Source/app`. GitHub Pages serves the repository at `/Source/`, so the page
 * and the iframe behind it share that base.
 */
export default defineConfig({
    root: fileURLToPath(new URL('.', import.meta.url)),
    base: '/Source/',
    plugins: [svelte()],

    server: {
        // 5173 is the Tauri dev server and 5174 the demo, so the site takes the next one.
        port: 5175,
        strictPort: true,
        // Keep the framed app on this origin in dev too: the site and the demo
        // then share `source-locale` and `immich-ui-theme` exactly as in Pages.
        proxy: {
            '/Source/app': {target: 'http://localhost:5174', changeOrigin: false, ws: true},
        },
    },

    build: {target: 'es2022'},
});
