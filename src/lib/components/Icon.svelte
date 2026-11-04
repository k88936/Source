<script lang="ts">
    type IconName =
        | 'photos'
        | 'search'
        | 'albums'
        | 'library'
        | 'settings'
        | 'sync'
        | 'heart'
        | 'heart-filled'
        | 'close'
        | 'back'
        | 'forward'
        | 'download'
        | 'upload'
        | 'play'
        | 'check'
        | 'check-circle'
        | 'more'
        | 'cloud'
        | 'folder'
        | 'image'
        | 'video'
        | 'logout'
        | 'trash'
        | 'alert'
        | 'info'
        | 'grid'
        | 'select'
        | 'eye'
        | 'chevron-up'
        | 'calendar'
        | 'stop'
        | 'wifi-off';

    let {name, size = 22, class: className = ''}: {name: IconName; size?: number; class?: string} = $props();

    const paths: Record<IconName, string> = {
        photos: 'M4 5h16v14H4z M4 15l4-4 4 4 3-3 5 5 M15 8.5a1.5 1.5 0 1 0 0 .01',
        search: 'M10.5 18a7.5 7.5 0 1 0 0-15 7.5 7.5 0 0 0 0 15Z M16 16l5 5',
        albums: 'M3 7h7l2 2h9v11H3z M3 7V5h7l2 2',
        library: 'M4 4h6v16H4z M14 4h6v16h-6z M14 8h6',
        settings:
            'M12 15.5a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7Z M19 12a7 7 0 0 0-.1-1.2l2-1.5-2-3.4-2.4 1a7 7 0 0 0-2-1.2L14 3h-4l-.5 2.7a7 7 0 0 0-2 1.2l-2.4-1-2 3.4 2 1.5A7 7 0 0 0 5 12c0 .4 0 .8.1 1.2l-2 1.5 2 3.4 2.4-1a7 7 0 0 0 2 1.2L10 21h4l.5-2.7a7 7 0 0 0 2-1.2l2.4 1 2-3.4-2-1.5c.1-.4.1-.8.1-1.2Z',
        sync: 'M20 11a8 8 0 0 0-13.7-5.6L4 8 M4 4v4h4 M4 13a8 8 0 0 0 13.7 5.6L20 16 M20 20v-4h-4',
        heart: 'M12 20s-7-4.4-7-9.2A4 4 0 0 1 12 8a4 4 0 0 1 7 2.8C19 15.6 12 20 12 20Z',
        'heart-filled': 'M12 20s-7-4.4-7-9.2A4 4 0 0 1 12 8a4 4 0 0 1 7 2.8C19 15.6 12 20 12 20Z',
        close: 'M6 6l12 12 M18 6 6 18',
        back: 'M15 5l-7 7 7 7',
        forward: 'M9 5l7 7-7 7',
        download: 'M12 3v12 M7 10l5 5 5-5 M4 21h16',
        upload: 'M12 16V4 M7 9l5-5 5 5 M4 20h16',
        play: 'M8 5l11 7-11 7Z',
        check: 'M4 12l5 5L20 6',
        'check-circle': 'M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18Z M8 12l3 3 5-6',
        more: 'M5 12h.01 M12 12h.01 M19 12h.01',
        cloud: 'M7 18a4 4 0 0 1-.4-8A6 6 0 0 1 18 9.5 4.5 4.5 0 0 1 17.5 18Z',
        folder: 'M3 6h7l2 2h9v11H3z',
        image: 'M4 5h16v14H4z M4 15l4-4 4 4 3-3 5 5 M15 8.5a1.5 1.5 0 1 0 0 .01',
        video: 'M3 6h13v12H3z M16 10l5-3v10l-5-3',
        logout: 'M9 4H4v16h5 M14 8l4 4-4 4 M8 12h10',
        trash: 'M4 7h16 M9 7V4h6v3 M6 7l1 14h10l1-14',
        alert: 'M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18Z M12 7v6 M12 17h.01',
        info: 'M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18Z M12 10v6 M12 7h.01',
        grid: 'M4 4h7v7H4z M13 4h7v7h-7z M4 13h7v7H4z M13 13h7v7h-7z',
        select: 'M4 4h6v6H4z M14 4h6v6h-6z M4 14h6v6H4z M14 14h6v6h-6z',
        eye: 'M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12Z M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6Z',
        'chevron-up': 'M6 15l6-6 6 6',
        calendar: 'M4 5h16v15H4z M4 9h16 M8 3v4 M16 3v4',
        stop: 'M7 7h10v10H7z',
        'wifi-off': 'M2 2l20 20 M8.5 8.5A10 10 0 0 1 22 12 M2 12a10 10 0 0 1 4-3 M12 20h.01',
    };
</script>

<svg
    width={size}
    height={size}
    viewBox="0 0 24 24"
    fill={name === 'heart-filled' || name === 'play' ? 'currentColor' : 'none'}
    stroke="currentColor"
    stroke-width="1.8"
    stroke-linecap="round"
    stroke-linejoin="round"
    class={className}
    aria-hidden="true"
>
    <path d={paths[name]} />
</svg>
