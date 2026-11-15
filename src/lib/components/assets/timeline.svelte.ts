import type {AssetSummary} from '#lib/api/bindings/AssetSummary';
import type {TimelineCursor} from '#lib/api/bindings/TimelineCursor';
import type {TimelinePage} from '#lib/api/bindings/TimelinePage';
import {app} from '#lib/app/app.svelte';
import type {Listeners, Subscription} from 'subscription-client';

/** How many rows one screen asks for, and how far below its last row the next anchor is looked for. */
export const PAGE_SIZE = 80;

export type PageLoader = (cursor: TimelineCursor | null, limit: number) => Promise<TimelinePage>;

/** Where the key `depth` rows below `after` is, or null when the scope ends first. */
export type EdgeLoader = (after: TimelineCursor | null, depth: number) => Promise<TimelineCursor | null>;

/** One interval of the timeline: the rows between two anchors. */
export type WindowRange = {
    from: TimelineCursor | null;
    to: TimelineCursor | null;
};

/** Opens the interval a range describes; an absent anchor is that end of the timeline. */
export type WindowOpener = (range: WindowRange, listeners: Listeners) => Promise<Subscription<AssetSummary>>;

/** How many rows the scope holds, as the live row says it, for the counter under the viewer. */
export type CountLoader = () => number;

/**
 * Where a list's rows come from.
 *
 * A catalog list is a stack of live intervals: the first screen is the newest
 * rows, and scrolling asks the catalog where the next screen ends and holds the
 * interval in between. Search is a plain page query, because its conditions are
 * not a view the catalog declares.
 */
export type TimelineSource = {page: PageLoader} | {window: WindowOpener; edge: EdgeLoader; count: CountLoader};

/** One interval held open, and the rows inside it as the catalog last sent them. */
type Window = WindowRange & {
    rows: AssetSummary[];
    /** Which window this is, so an answer that arrives late lands in the interval it was asked for. */
    cut: number;
};

/**
 * Rows one window may hold before its anchors are cut again.
 *
 * A window is cut for one screen (`PAGE_SIZE`) and grows when the scope gains
 * rows inside its range. The cap is loose on purpose: cutting drops the windows
 * below, so a few hundred new photos must not cost the user what they scrolled.
 */
const MAX_WINDOW_ROWS = 256;

/**
 * Newest-first asset list shared by photos / album / search.
 *
 * A subscribed list holds one interval per screenful the user has reached, so a
 * commit reaches the rows already on screen instead of asking for them again.
 * The rows are read from memory; only an anchor per screenful and a page of the
 * search results ever reach the catalog.
 */
export class Timeline {
    /** Every interval opened so far, head first; each holds exactly its own rows. */
    private windows = $state.raw<Window[]>([]);
    /** Rows a plain page query brought in, behind the windows. */
    private pages = $state.raw<AssetSummary[]>([]);
    /** Where the next page of a plain query starts. */
    private cursor = $state<TimelineCursor | null>(null);
    /** The deepest interval reaches the end of the scope. */
    private exhausted = $state(false);
    total = $state(0);
    loading = $state(false);
    loaded = $state(false);
    error = $state('');

    private source: TimelineSource;
    /** The live interval behind every open window, by the window it feeds. */
    private live = new Map<number, Subscription<AssetSummary>>();
    private nextCut = 0;
    private requestId = 0;

    constructor(source: TimelineSource) {
        this.source = source;
    }

    /** The list on screen, newest first, each row once. */
    get items(): AssetSummary[] {
        const rows: AssetSummary[] = [];
        const held = new Set<string>();
        for (const window of this.windows) {
            for (const row of window.rows) {
                if (held.has(row.id)) continue;
                held.add(row.id);
                rows.push(row);
            }
        }
        for (const row of this.pages) {
            if (held.has(row.id)) continue;
            held.add(row.id);
            rows.push(row);
        }
        return rows;
    }

    /** Another screenful exists below the last row held. */
    get hasMore(): boolean {
        return 'page' in this.source ? this.cursor !== null : !this.exhausted;
    }

    /** Re-read the scope's length; the rows themselves arrive as deltas. */
    refreshTotal(): void {
        if (!('window' in this.source)) return;
        this.total = this.source.count();
    }

    /** Swap the query a plain, paged list runs, e.g. when the search filters change. */
    setPageLoader(page: PageLoader): void {
        this.source = {page};
    }

    /** Start over with the first screen, or the first page of a plain query. */
    async reload(): Promise<void> {
        const id = ++this.requestId;
        this.stop();
        this.loading = true;
        this.error = '';
        try {
            if ('page' in this.source) {
                const page = await this.source.page(null, PAGE_SIZE);
                if (id !== this.requestId) return;
                this.pages = page.items;
                this.cursor = page.next;
                this.total = page.total;
            } else {
                await this.#open(id);
                if (id !== this.requestId) return;
                this.refreshTotal();
            }
            this.loaded = true;
        } catch (err) {
            if (id !== this.requestId) return;
            this.error = app.describeError(err);
            this.loaded = true;
        } finally {
            if (id === this.requestId) this.loading = false;
        }
    }

    /** Reach one screenful further down. */
    async loadMore(): Promise<void> {
        if (this.loading || !this.hasMore) return;
        const id = this.requestId;
        this.loading = true;
        try {
            if ('page' in this.source) {
                const page = await this.source.page(this.cursor, PAGE_SIZE);
                if (id !== this.requestId) return;
                this.pages = [...this.pages, ...page.items];
                this.cursor = page.next;
                this.total = page.total;
            } else {
                await this.#open(id);
            }
        } catch (err) {
            if (id === this.requestId) this.error = app.describeError(err);
        } finally {
            if (id === this.requestId) this.loading = false;
        }
    }

    /** Insert or replace one row, keeping the newest-first order of its window. */
    upsert(asset: AssetSummary): void {
        if (this.pages.some((row) => row.id === asset.id)) {
            this.pages = this.pages.map((row) => (row.id === asset.id ? asset : row));
            return;
        }
        this.windows = this.windows.map((window) =>
            window.rows.some((row) => row.id === asset.id)
                ? {
                      ...window,
                      rows: window.rows.map((row) => (row.id === asset.id ? asset : row)),
                  }
                : window,
        );
    }

    /**
     * Drop rows a move took out of the scope, before their window says so.
     *
     * The length is not adjusted here: it is a view row the server recomputes,
     * and a commit that moves an asset sends the new total anyway.
     */
    remove(ids: string[]): void {
        const drop = new Set(ids);
        this.pages = this.pages.filter((row) => !drop.has(row.id));
        this.windows = this.windows.map((window) => ({
            ...window,
            rows: window.rows.filter((row) => !drop.has(row.id)),
        }));
    }

    reset(): void {
        this.requestId += 1;
        this.stop();
        this.loaded = false;
        this.loading = false;
        this.error = '';
    }

    /** Drop every open interval and the rows they brought. */
    stop(): void {
        for (const subscription of this.live.values()) subscription.stop();
        this.live.clear();
        this.windows = [];
        this.pages = [];
        this.cursor = null;
        this.exhausted = false;
        this.total = 0;
    }

    /**
     * Open the next interval down and hold it until the list is reset or cut.
     *
     * The interval starts where the previous one stopped and ends `PAGE_SIZE`
     * keys below it, so the catalog sends one screenful and the two intervals
     * neither overlap nor leave a hole. A missing end means the scope has less
     * than a screenful left, so this interval runs to the oldest row and there
     * is nothing below it. A scope that is only short right now answers the same
     * way, which is what `#cut` watches for.
     */
    async #open(id: number): Promise<void> {
        if (!('window' in this.source)) return;
        const source = this.source;
        const deepest = this.windows[this.windows.length - 1];
        const to = deepest ? deepest.from : null;
        const from = await source.edge(to, PAGE_SIZE);
        if (id !== this.requestId) return;
        const cut = ++this.nextCut;
        this.windows = [...this.windows, {from, to, rows: [], cut}];
        const at = () => this.windows.findIndex((window) => window.cut === cut);
        // The window answers its first snapshot before `subscribe` resolves, so a
        // commit that lands during opening has no rows to show yet.
        const subscription = await source.window(
            {from, to},
            {
                changed: () => {
                    // A commit for a list that was reset in the meantime, or for a
                    // window a cut already dropped, has no interval to land in.
                    if (id !== this.requestId) return;
                    const rows = this.live.get(cut)?.rows();
                    const index = at();
                    if (!rows || index < 0) return;
                    this.#replace(index, rows);
                    this.#cut(index, id);
                },
                failed: (message) => {
                    if (id === this.requestId) this.error = message;
                },
            },
        );
        const index = at();
        if (id !== this.requestId || index < 0) {
            subscription.stop();
            return;
        }
        this.live.set(cut, subscription);
        this.#replace(index, subscription.rows());
        this.exhausted = from === null;
    }

    /**
     * Cut the list again from the window whose range went stale.
     *
     * An interval is a condition the catalog applies, chosen from where one screen
     * ended when the window was opened. A scope with less than a screenful answers
     * that question with no anchor at all, so the window runs to the oldest row;
     * when the scope then gains rows - an album indexing into a list that held
     * three - every one of them is inside it, and one screen turns into the whole
     * library. Dropping rows here would not bound it: the next commit brings them
     * back, because they are still inside the interval. Only a new anchor does, so
     * the window is opened again, and so are the windows below it, whose upper
     * anchor was this window's.
     */
    #cut(index: number, id: number): void {
        if (this.windows[index]!.rows.length <= MAX_WINDOW_ROWS) return;
        const dropped = this.windows.slice(index);
        this.windows = this.windows.slice(0, index);
        for (const window of dropped) {
            this.live.get(window.cut)?.stop();
            this.live.delete(window.cut);
        }
        void this.#open(id).catch((err) => {
            if (id === this.requestId) this.error = app.describeError(err);
        });
    }

    #replace(index: number, rows: AssetSummary[]): void {
        this.windows = this.windows.map((held, at) => (at === index ? {...held, rows} : held));
    }
}
