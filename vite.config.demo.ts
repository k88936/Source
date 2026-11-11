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

    optimizeDeps: {exclude: ['svelte-navigation', '@immich/ui']},

    resolve: {
        alias: [
            // `@immich/ui` is written for SvelteKit and imports `$app/*`; the
            // demo is a plain Vite build too, so the same shims apply.
            {find: '$app/env', replacement: resolve('src/lib/hack/svelteKit4immich/env.ts')},
            {find: '$app/navigation', replacement: resolve('src/lib/hack/svelteKit4immich/navigation.ts')},
            {find: '$app/state', replacement: resolve('src/lib/hack/svelteKit4immich/state.ts')},
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
        // Same excludes as `vite.config.ts`; this server shares the project root.
        watch: {
            ignored: [
                '**/target/**',
                '**/src-tauri/**',
                '**/tmp/**',
                '**/.junie/**',
                '**/.playwright-cli/**',
                '**/dist-demo/**',
                '**/.awb/**',
            ],
        },
    },
});
