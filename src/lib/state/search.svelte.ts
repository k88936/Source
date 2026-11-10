import {tick} from 'svelte';
import {api} from '#lib/api/client';
import type {MediaType} from '#lib/api/bindings/MediaType';
import type {TimelineCursor} from '#lib/api/bindings/TimelineCursor';
import type {TimelinePage} from '#lib/api/bindings/TimelinePage';
import type {Selection} from './selection.svelte';
import {Timeline} from './timeline.svelte';

const RECENT_KEY = 'recent-searches';
const RECENT_LIMIT = 8;
const DEBOUNCE_MS = 280;

/** The conditions taken as one value, so a debounced run cannot mix two edits. */
type Filters = {
    query: string;
    mediaType: MediaType | null;
    favorite: boolean;
    from: string;
    to: string;
};

function dayStart(value: string): number | null {
    return value ? new Date(`${value}T00:00:00`).getTime() : null;
}

function dayEnd(value: string): number | null {
    return value ? new Date(`${value}T23:59:59.999`).getTime() : null;
}

function readRecent(): string[] {
    try {
        const parsed: unknown = JSON.parse(localStorage.getItem(RECENT_KEY) ?? '[]');
        return Array.isArray(parsed) ? parsed.filter((item): item is string => typeof item === 'string') : [];
    } catch {
        return [];
    }
}

/**
 * Search as a mode of one page rather than a destination of its own: the grid
 * and the selection stay the page's, and only the list underneath is swapped.
 *
 * Conditions live as long as the page does. Leaving the mode keeps them, so the
 * mode reopens where it was left; leaving the page drops them; they never reach
 * the URL. The recent terms are the one thing that persists between sessions.
 */
export class Search {
    open = $state(false);
    query = $state('');
    mediaType = $state<MediaType | null>(null);
    favorite = $state(false);
    from = $state('');
    to = $state('');
    /** A run has finished, so the idle text steps aside for the results. */
    started = $state(false);
    recent = $state<string[]>(readRecent());
    readonly timeline: Timeline;

    /** Album the results are confined to, or `null` for the whole library. */
    private folder: () => string | null;
    private selection: Selection;
    private timer: ReturnType<typeof setTimeout> | undefined;
    /** The page's scroll container, so the mode reopens where it was left. */
    private scroll: () => HTMLElement | undefined;
    private scrollY = 0;

    constructor(
        folder: () => string | null,
        selection: Selection,
        scroll: () => HTMLElement | undefined,
    ) {
        this.folder = folder;
        this.selection = selection;
        this.scroll = scroll;
        // Replaced by `run()` before the first request; an idle mode loads nothing.
        this.timeline = new Timeline((cursor, limit) =>
            this.request({query: '', mediaType: null, favorite: false, from: '', to: ''}, cursor, limit),
        );
    }

    /** True while some condition narrows the library; an empty mode is idle. */
    get filtered(): boolean {
        return Boolean(this.query.trim() || this.mediaType || this.favorite || this.from || this.to);
    }

    enter(): void {
        if (this.open) return;
        this.selection.exit();
        this.scrollY = this.scroll()?.scrollTop ?? 0;
        this.open = true;
        if (this.filtered) void this.run();
    }

    /** Leaving the mode is a plain close: the back key is the page's handler. */
    exit(): void {
        if (!this.open) return;
        this.close();
    }

    /** Forget the mode without touching anything; the page itself is going away. */
    dispose(): void {
        clearTimeout(this.timer);
    }

    setQuery(value: string): void {
        this.query = value;
        this.schedule();
    }

    setMediaType(value: MediaType | null): void {
        this.mediaType = value;
        this.schedule();
    }

    toggleFavorite(): void {
        this.favorite = !this.favorite;
        this.schedule();
    }

    setRange(from: string, to: string): void {
        this.from = from;
        this.to = to;
        this.schedule();
    }

    clearFilters(): void {
        this.query = '';
        this.mediaType = null;
        this.favorite = false;
        this.from = '';
        this.to = '';
        this.schedule();
    }

    applyRecent(term: string): void {
        this.setQuery(term);
    }

    /** Enter only: a debounced run would fill the list with every prefix. */
    remember(): void {
        const term = this.query.trim();
        if (!term) return;
        this.recent = [term, ...this.recent.filter((item) => item !== term)].slice(0, RECENT_LIMIT);
        localStorage.setItem(RECENT_KEY, JSON.stringify(this.recent));
    }

    /** Runs the current conditions now; the debounce and Retry both land here. */
    async run(): Promise<void> {
        const filters = this.take();
        if (!filters) {
            this.started = false;
            this.timeline.reset();
            return;
        }
        this.timeline.setLoader((cursor, limit) => this.request(filters, cursor, limit));
        await this.timeline.reload();
        this.started = true;
    }

    private request(filters: Filters, cursor: TimelineCursor | null, limit: number): Promise<TimelinePage> {
        return api.searchAssets({
            query: filters.query.trim(),
            folderId: this.folder(),
            from: dayStart(filters.from),
            to: dayEnd(filters.to),
            favorite: filters.favorite ? true : null,
            mediaType: filters.mediaType,
            cursor,
            limit,
        });
    }

    private take(): Filters | null {
        if (!this.filtered) return null;
        return {
            query: this.query,
            mediaType: this.mediaType,
            favorite: this.favorite,
            from: this.from,
            to: this.to,
        };
    }

    private schedule(): void {
        clearTimeout(this.timer);
        if (!this.open) return;
        this.timer = setTimeout(() => void this.run(), DEBOUNCE_MS);
    }

    private close(): void {
        this.open = false;
        this.selection.exit();
        this.dispose();
        void this.restore();
    }

    /** Back to where the timeline was left; the swap lands on the next frame. */
    private async restore(): Promise<void> {
        await tick();
        this.scroll()?.scrollTo(0, this.scrollY);
    }
}
