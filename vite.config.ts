import {mkdirSync, writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {svelte} from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';
import {defineConfig, type Plugin} from 'vite';

const host = process.env.TAURI_DEV_HOST;

// Kept in sync with the dev build, which reads `<repo>/tmp/vite.json`.
const devTmp = process.env.SOURCE_DEV_TMP ?? resolve(process.cwd(), 'tmp');

/**
 * Publish the dev server's URL so the app (and anything else running locally)
 * can find the frontend without hardcoding it. 1420 is only the preferred port:
 * when it is taken Vite moves on, and a dev build follows whatever lands in
 * this file instead of insisting on `build.devUrl`.
 */
const writeDevUrl = (): Plugin => ({
    name: 'source-dev-url',
    configureServer(server) {
        const publish = () => {
            const address = server.httpServer?.address();
            if (!address || typeof address === 'string') {
                return;
            }
            mkdirSync(devTmp, {recursive: true});
            const url = `http://localhost:${address.port}`;
            writeFileSync(resolve(devTmp, 'vite.json'), `${JSON.stringify({url}, null, 2)}\n`);
            server.config.logger.info(`dev url written to ${resolve(devTmp, 'vite.json')}`);
        };
        server.httpServer?.once('listening', publish);
    },
});

export default defineConfig({
    plugins: [tailwindcss(), svelte(), writeDevUrl()],

    // `@immich/ui` is written for SvelteKit and imports `$app/*`; the app is a
    // plain Vite build, so those virtual modules resolve to thin shims.
    resolve: {
        alias: {
            '$app/env': resolve(process.cwd(), 'src/lib/immich/env.ts'),
            '$app/navigation': resolve(process.cwd(), 'src/lib/immich/navigation.ts'),
            '$app/state': resolve(process.cwd(), 'src/lib/immich/state.ts'),
        },
    },

    // The navigator imports `.svelte.ts` state out of the linked package, which
    // Vite must compile as source instead of pre-bundling. The UI package needs
    // the same treatment so its `$app/*` imports go through the aliases.
    optimizeDeps: {exclude: ['svelte-navigation', '@immich/ui']},

    // prevent Vite from obscuring Rust errors
    clearScreen: false,
    build: {target: 'es2022', outDir: 'build'},
    server: {
        port: 1420,
        strictPort: false,
        host: host || false,
        hmr: host ? {protocol: 'ws', host, port: 1421} : undefined,
        watch: {ignored: ['**/src-tauri/**']},
    },
});
