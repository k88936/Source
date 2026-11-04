<script lang="ts">
    import {page} from '$app/state';
    import Icon from './Icon.svelte';

    const items = [
        {href: '/photos', label: '照片', icon: 'photos' as const},
        {href: '/search', label: '搜索', icon: 'search' as const},
        {href: '/albums', label: '相册', icon: 'albums' as const},
        {href: '/library', label: '资料库', icon: 'library' as const},
        {href: '/settings', label: '设置', icon: 'settings' as const},
    ];

    function active(href: string): boolean {
        return page.url.pathname === href || page.url.pathname.startsWith(href + '/');
    }
</script>

<!-- Desktop shell: the phone bottom bar would read as a stretched phone. -->
<aside class="fixed inset-y-0 left-0 z-30 hidden w-60 flex-col border-e border-line bg-light px-3 py-4 sidebar:flex">
    <a href="/photos" class="mb-6 flex items-center gap-3 px-2 py-1">
        <span class="grid size-9 place-items-center rounded-xl bg-primary text-on-primary">
            <Icon name="cloud" size={20} />
        </span>
        <span class="text-base font-semibold">Source</span>
    </a>

    <nav class="-mx-3 flex flex-col gap-0.5" aria-label="主导航">
        {#each items as item (item.href)}
            <a
                href={item.href}
                class="flex items-center gap-4 rounded-e-full py-2.5 pe-4 ps-5 text-sm font-medium transition-colors {active(
                    item.href,
                )
                    ? 'bg-primary/10 text-primary'
                    : 'text-muted hover:bg-gray hover:text-dark'}"
                aria-current={active(item.href) ? 'page' : undefined}
            >
                <Icon name={item.icon} size={20} />
                <span>{item.label}</span>
            </a>
        {/each}
    </nav>
</aside>
