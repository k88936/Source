import {i18n} from '#lib/i18n/i18n.svelte';

export function formatBytes(bytes: number): string {
    if (!Number.isFinite(bytes) || bytes <= 0) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    const exponent = Math.min(units.length - 1, Math.floor(Math.log(bytes) / Math.log(1024)));
    const value = bytes / 1024 ** exponent;
    return `${value >= 10 || exponent === 0 ? value.toFixed(0) : value.toFixed(1)} ${units[exponent]}`;
}

type Formatters = {date: Intl.DateTimeFormat; relative: Intl.RelativeTimeFormat};

// Rebuilt only when the locale changes: Intl construction is expensive and these run per asset.
let cached: {tag: string; value: Formatters} | null = null;

function formatters(): Formatters {
    if (cached?.tag === i18n.tag) return cached.value;
    cached = {
        tag: i18n.tag,
        value: {
            date: new Intl.DateTimeFormat(i18n.tag, {year: 'numeric', month: '2-digit', day: '2-digit'}),
            relative: new Intl.RelativeTimeFormat(i18n.tag, {numeric: 'auto'}),
        },
    };
    return cached.value;
}

export function formatDate(ms: number): string {
    return formatters().date.format(new Date(ms));
}

export function formatRelative(ms: number): string {
    const diff = Date.now() - ms;
    if (diff < 60_000) return formatters().relative.format(0, 'second');
    if (diff < 3_600_000) return formatters().relative.format(-Math.floor(diff / 60_000), 'minute');
    if (diff < 86_400_000) return formatters().relative.format(-Math.floor(diff / 3_600_000), 'hour');
    if (diff < 7 * 86_400_000) return formatters().relative.format(-Math.floor(diff / 86_400_000), 'day');
    return formatDate(ms);
}
