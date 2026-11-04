<script lang="ts">
    import {page} from '$app/state';
    import {i18n} from '#lib/state/i18n.svelte';
    import {nav} from '#lib/i18n/nav.i18n';
    import {Cloud, Image, Images, LibraryBig, Search, Settings} from '@lucide/svelte';

    const t = $derived(i18n.pick(nav));

    const items = $derived([
        {href: '/photos', label: t.photos, icon: Image},
        {href: '/search', label: t.search, icon: Search},
        {href: '/albums', label: t.albums, icon: Images},
        {href: '/library', label: t.library, icon: LibraryBig},
        {href: '/settings', label: t.settings, icon: Settings},
    ]);

    function active(href: string): boolean {
        return page.url.pathname === href || page.url.pathname.startsWith(href + '/');
    }
</script>

<!-- Desktop shell: the phone bottom bar would read as a stretched phone. -->
<aside class="fixed inset-y-0 left-0 z-30 hidden w-60 flex-col border-e border-line bg-light px-3 py-4 sidebar:flex">
    <a href="/photos" class="mb-6 flex items-center gap-3 px-2 py-1">
        <span class="grid size-9 place-items-center rounded-xl bg-primary text-on-primary">
            <Cloud size={20} />
        </span>
        <span class="text-base font-semibold">Source</span>
    </a>

    <nav class="-mx-3 flex flex-col gap-0.5" aria-label={t.ariaLabel}>
        {#each items as item (item.href)}
            {@const Icon = item.icon}
            <a
                href={item.href}
                class="flex items-center gap-4 rounded-e-full py-2.5 pe-4 ps-5 text-sm font-medium transition-colors {active(
                    item.href,
                )
                    ? 'bg-primary/10 text-primary'
                    : 'text-muted hover:bg-gray hover:text-dark'}"
                aria-current={active(item.href) ? 'page' : undefined}
            >
                <Icon size={20} />
                <span>{item.label}</span>
            </a>
        {/each}
    </nav>
</aside>
