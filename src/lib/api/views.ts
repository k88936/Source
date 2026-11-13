import type {AlbumMetadata} from '#lib/api/bindings/AlbumMetadata';
import type {AssetSummary} from '#lib/api/bindings/AssetSummary';
import type {AssetsQuery} from '#lib/api/bindings/AssetsQuery';
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
const assets = new Client<AssetsQuery, AssetSummary>(wireFor('assets'));

const byName = (a: AlbumMetadata, b: AlbumMetadata) => a.name.localeCompare(b.name);
const newestFirst = (a: AssetSummary, b: AssetSummary) => b.takenAt - a.takenAt || b.id.localeCompare(a.id);

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
