import {resolve} from 'node:path';
import {svelte} from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';
import {defineConfig} from 'vite';

/**
 * The demo build: the same app, published under `/Source/app` and wired to the
 * fixture transport in `src/demo`. It never touches the Tauri build, which keeps
 * `vite.config.ts` and an empty base of its own.
 */
const base = '/Source/app';

export default defineConfig({
    base,
    plugins: [tailwindcss(), svelte()],

    optimizeDeps: {exclude: ['svelte-navigation']},

    resolve: {
        alias: [
            {find: /^#lib\/api\/transport$/, replacement: resolve('src/demo/transport.ts')},
            {
                find: /^@saurl\/tauri-plugin-safe-area-insets-css-api$/,
                replacement: resolve('src/demo/safe-area.ts'),
            },
        ],
    },

    // Two configs over one source: sharing Vite's optimizer cache makes both dev
    // servers answer "Outdated Optimize Dep" at each other.
    cacheDir: 'node_modules/.vite-demo',

    clearScreen: false,
    build: {target: 'es2022', outDir: 'dist-demo'},
    server: {
        // The site proxies this port under its own origin, so the port is fixed.
        port: 5174,
        strictPort: true,
        watch: {ignored: ['**/src-tauri/**']},
    },
});
