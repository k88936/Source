import type {AlbumMetadata} from '#lib/api/bindings/AlbumMetadata';
import type {AssetSummary} from '#lib/api/bindings/AssetSummary';
import type {AssetTotal} from '#lib/api/bindings/AssetTotal';
import type {AssetsQuery} from '#lib/api/bindings/AssetsQuery';
import type {JobRow} from '#lib/api/bindings/JobRow';
import type {LocalAlbumMetadata} from '#lib/api/bindings/LocalAlbumMetadata';
import type {QueuedUpload} from '#lib/api/bindings/QueuedUpload';
import type {TimelineCursor} from '#lib/api/bindings/TimelineCursor';
import type {Listeners, Subscription} from 'subscription-client';
import {errorText} from '#lib/errors/i18n';
import {wireFor} from '#lib/api/subscription';
import {Client, useQuery} from 'subscription-client';

/**
 * The album grid's query: it takes no parameters, and `null` is how the wire
 * carries a query with nothing to name.
 *
 * Rust declares the same empty parameter type, so the two sides agree on the
 * shape without a tag: which view is being asked is the connection, not a field.
 */
export type AlbumsQuery = null;

/** One interval of the timeline, as a page asks for it. */
type Window = {
    albumId: string | null;
    from: TimelineCursor | null;
    to: TimelineCursor | null;
};

/**
 * One client per view, each with its own connection and its own copy of rows.
 *
 * Rust answers each view from its own manager, so the two streams are
 * independent: a snapshot that walks the album table cannot hold up a window's
 * deltas, and a page that stops reading one stream cannot starve the other.
 */
const albums = new Client<AlbumsQuery, AlbumMetadata>(wireFor('albums'));
const locals = new Client<AlbumsQuery, LocalAlbumMetadata>(wireFor('locals'));
const assets = new Client<AssetsQuery, AssetSummary>(wireFor('assets'));
const jobs = new Client<AlbumsQuery, JobRow>(wireFor('jobs'));
const running = new Client<AlbumsQuery, QueuedUpload>(wireFor('running'));
const counts = new Client<AlbumsQuery, AssetTotal>(wireFor('counts'));

const byName = (a: {name: string}, b: {name: string}) => a.name.localeCompare(b.name);
const byId = (a: {id: string}, b: {id: string}) => a.id.localeCompare(b.id);
const newestFirst = (a: AssetSummary, b: AssetSummary) => b.takenAt - a.takenAt || b.id.localeCompare(a.id);
const newestJobFirst = (a: JobRow, b: JobRow) => b.submittedAt - a.submittedAt || a.id.localeCompare(b.id);

/**
 * The album grid: every album the user added, with its tile.
 *
 * A tile is derived on the server - a count, a mosaic, the outcome of the last
 * listing - so a commit re-sends the tiles it touched and the page never counts
 * anything itself.
 */
export const albumsView = {
    /** The grid, live for as long as the calling component is mounted. */
    use() {
        return useQuery(albums, {query: null, compare: byName}, errorText);
    },
};

/**
 * The local albums: the device directories syncing into the grid.
 *
 * The count and the mosaic are asked of the device every time a commit names
 * one of these rows, so the page never counts a folder itself.
 */
export const localsView = {
    /** The sources, live for as long as the calling component is mounted. */
    use() {
        return useQuery(locals, {query: null, compare: byName}, errorText);
    },
};

/**
 * The timeline: a stack of intervals, each held live on its own.
 *
 * An interval is decided row by row, so a row that crosses an anchor appears in
 * one and leaves the other; nothing here filters or truncates. The order is the
 * declaration's, and the client sorts its copy of the rows.
 */
export const assetsView = {
    /** One interval, held until the caller stops it. */
    watch(window: Window, listeners: Listeners): Promise<Subscription<AssetSummary>> {
        return assets.subscribe(
            {
                query: {albumId: window.albumId, from: window.from, to: window.to},
                compare: newestFirst,
            },
            listeners,
        );
    },
};

/**
 * The job queue: one row per job this process ran, newest first.
 *
 * A row is the queue's own state, so "this album is being listed" is a row here
 * and not a guess made from byte counters. The shell holds it for the lifetime
 * of the window, because cards across the app read it.
 */
export const jobsView = {
    /** The queue, live until the caller stops it. */
    watch(listeners: Listeners): Promise<Subscription<JobRow>> {
        return jobs.subscribe({query: null, compare: newestJobFirst}, listeners);
    },
};

/**
 * The uploads in flight: only the rows whose bytes are moving.
 *
 * A queued upload waits its turn with nothing to show and a finished one is
 * history, so the set is bounded by the concurrency instead of by the backlog.
 */
export const runningView = {
    /** The moving rows, live for as long as the calling component is mounted. */
    use() {
        return useQuery(running, {query: null, compare: byName}, errorText);
    },
};

/**
 * The length of the library: how many assets the catalog holds.
 *
 * A window only holds the intervals a page scrolled to, so the length is its own
 * row. The server recomputes it on every commit that writes an asset, so a page
 * prints it without ever counting a page of its own.
 */
export const countsView = {
    /** The total, live for as long as the calling component is mounted. */
    use() {
        return useQuery(counts, {query: null, compare: byId}, errorText);
    },
};
