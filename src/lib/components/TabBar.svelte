<script lang="ts">
    import {Image, Images, LibraryBig, Search} from '@lucide/svelte';
    import {resolve} from '$app/paths';
    import {page} from '$app/state';
    import {i18n} from '#lib/state/i18n.svelte';
    import {nav} from '#lib/i18n/nav.i18n';

    const t = $derived(i18n.pick(nav));

    const tabs = $derived([
        {href: resolve('/photos'), label: t.photos, icon: Image},
        {href: resolve('/search'), label: t.search, icon: Search},
        {href: resolve('/albums'), label: t.albums, icon: Images},
        {href: resolve('/library'), label: t.library, icon: LibraryBig},
    ]);

    function active(href: string): boolean {
        return page.url.pathname === href || page.url.pathname.startsWith(href + '/');
    }
</script>

<!-- Phone shell only: from `sidebar` up, navigation moves into the side rail. -->
<nav
    class="fixed inset-x-0 bottom-0 z-30 border-t border-line bg-light/95 pb-[var(--safe-bottom)] backdrop-blur-xl sidebar:hidden"
    aria-label={t.ariaLabel}
>
    <div class="mx-auto flex max-w-3xl items-stretch">
        {#each tabs as tab (tab.href)}
            {@const Icon = tab.icon}
            <a
                href={tab.href}
                class="flex flex-1 flex-col items-center gap-1 py-2 text-[11px] font-medium transition-colors {active(
                    tab.href,
                )
                    ? 'text-primary'
                    : 'text-muted'}"
                aria-current={active(tab.href) ? 'page' : undefined}
            >
                <Icon size={23} />
                <span>{tab.label}</span>
            </a>
        {/each}
    </div>
</nav>
