/**
 * Stand-in for SvelteKit's `$app/navigation`. `goto` only backs internal links
 * in the package's utilities and a full load is the correct behaviour here; the
 * app owns its navigation stack, so no component listens for `afterNavigate`.
 */
export function goto(href: string, options?: {replaceState?: boolean}): Promise<void> {
    if (options?.replaceState) {
        window.location.replace(href);
    } else {
        window.location.assign(href);
    }
    return Promise.resolve();
}

export function afterNavigate(_callback?: unknown): void {
    // The app navigates through `svelte-navigation`, not through Kit.
}
