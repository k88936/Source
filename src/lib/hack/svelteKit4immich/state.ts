/**
 * Stand-in for SvelteKit's `$app/state`. Only `NavbarItem` reads `page.url`,
 * and the app's top bar does not use that component, so a current-URL snapshot
 * is enough to satisfy the import.
 */
export const page = {
    url: new URL(window.location.href),
    params: {} as Record<string, string>,
    route: {id: null as string | null},
    status: 200,
    error: null,
    data: {},
    state: {},
};
