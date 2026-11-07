import {resolve} from 'node:path';
import adapter from '@sveltejs/adapter-static';
import {sveltekit} from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import {defineConfig} from 'vite';

/**
 * The demo build: the same app, published under `/Source/app` and wired to the
 * fixture transport in `src/demo`. It never touches the Tauri build, which keeps
 * `vite.config.ts` and an empty base of its own.
 */
const base = '/Source/app';

export default defineConfig({
    plugins: [
        tailwindcss(),
        sveltekit({
            adapter: adapter({pages: 'dist-demo', assets: 'dist-demo', fallback: 'index.html', precompress: false}),
            paths: {base, relative: false},
        }),
    ],

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
    build: {target: 'es2022'},
    server: {
        // The site proxies this port under its own origin, so the port is fixed.
        port: 5174,
        strictPort: true,
        watch: {ignored: ['**/src-tauri/**']},
    },
});
