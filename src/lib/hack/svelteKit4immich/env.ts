/**
 * Stand-in for SvelteKit's `$app/env`, which `@immich/ui` imports at runtime.
 * The package ships one entry that pulls in every component, so Vite resolves
 * these names even when the app uses none of the components that need them.
 */
export const browser = typeof window !== 'undefined';
