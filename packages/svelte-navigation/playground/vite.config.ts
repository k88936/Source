import {fileURLToPath} from 'node:url';
import {svelte} from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';
import {defineConfig} from 'vite';

/**
 * The package's own playground: `pnpm nav:dev` then open it in a browser. It is
 * the only place the navigator can be tried without the app around it.
 */
export default defineConfig({
    root: fileURLToPath(new URL('.', import.meta.url)),
    plugins: [tailwindcss(), svelte()],
    clearScreen: false,
    build: {target: 'es2022'},
    server: {
        port: 5176,
        strictPort: true,
    },
});
