<script lang="ts">
    import {page} from '$app/state';
    import Icon from './Icon.svelte';

    const tabs = [
        {href: '/photos', label: '照片', icon: 'photos' as const},
        {href: '/search', label: '搜索', icon: 'search' as const},
        {href: '/albums', label: '相册', icon: 'albums' as const},
        {href: '/library', label: '资料库', icon: 'library' as const},
    ];

    function active(href: string): boolean {
        return page.url.pathname === href || page.url.pathname.startsWith(href + '/');
    }
</script>

<!-- Phone shell only: from `sidebar` up, navigation moves into the side rail. -->
<nav
    class="fixed inset-x-0 bottom-0 z-30 border-t border-line bg-light/95 pb-[var(--safe-bottom)] backdrop-blur-xl sidebar:hidden"
    aria-label="主导航"
>
    <div class="mx-auto flex max-w-3xl items-stretch">
        {#each tabs as tab (tab.href)}
            <a
                href={tab.href}
                class="flex flex-1 flex-col items-center gap-1 py-2 text-[11px] font-medium transition-colors {active(
                    tab.href,
                )
                    ? 'text-primary'
                    : 'text-muted'}"
                aria-current={active(tab.href) ? 'page' : undefined}
            >
                <Icon name={tab.icon} size={23} />
                <span>{tab.label}</span>
            </a>
        {/each}
    </div>
</nav>
