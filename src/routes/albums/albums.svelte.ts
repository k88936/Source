import type {AlbumMetadata} from '#lib/api/bindings/AlbumMetadata';
import type {JobKind} from '#lib/api/bindings/JobKind';
import type {JobRow} from '#lib/api/bindings/JobRow';
import type {LocalAlbumMetadata} from '#lib/api/bindings/LocalAlbumMetadata';
import type {QueuedUpload} from '#lib/api/bindings/QueuedUpload';
import {api} from '#lib/api/client';
import {albumsView, localsView, runningView} from '#lib/api/views';
import {app} from '#lib/app/app.svelte';

/// How a card reads at a glance. A listing or a scan has a state of its own,
/// so a card moves while it works and before a single byte is on the wire.
export type Tone = 'idle' | 'scan' | 'upload';

/// One device folder stacked beside the album it uploads into.
export interface Mapping {
    local: LocalAlbumMetadata;
    tone: Tone;
    /// The scan job for this folder, while there is one.
    job: JobRow | null;
    /// The files of this folder whose bytes are moving right now.
    tasks: QueuedUpload[];
}

/// One album and the folders that sync into it.
export interface Group {
    album: AlbumMetadata;
    tone: Tone;
    /// The listing job for this album, while there is one.
    job: JobRow | null;
    mappings: Mapping[];
}

/// Whether a job is still going to do something for its card.
function live(job: JobRow | undefined): boolean {
    return job?.state === 'queued' || job?.state === 'running';
}

/**
 * The graph's data: what is mapped where, what each row is moving, and the
 * actions that change a mapping.
 *
 * The page and the sheet both read one instance, so a folder opened in a sheet
 * is answered by the same rows the card is drawn from.
 */
export class SyncBoard {
    albums = $state<AlbumMetadata[]>([]);
    locals = $state<LocalAlbumMetadata[]>([]);
    /// The uploads whose bytes are moving. Only those: a queued upload has
    /// nothing to show and a finished one is history, so the set is bounded by
    /// the upload concurrency instead of by the backlog.
    tasks = $state<QueuedUpload[]>([]);

    /// One cloud album drives a row: it is the destination, the local folders
    /// are the sources stacked beside it, and an album nobody points at still
    /// gets its row because it is still part of the library.
    ///
    /// Every moving upload carries the folder that asked for it, so a folder is
    /// answered from its own rows and an album from their sum.
    groups: Group[] = $derived.by(() => {
        const scans = new Map<string, JobRow>();
        const listings = new Map<string, JobRow>();
        for (const job of app.jobs) {
            if (job.kind === 'sync') scans.set(job.jobId, job);
            else if (job.kind === 'index') listings.set(job.jobId, job);
        }

        const byAlbum = new Map<string, LocalAlbumMetadata[]>();
        for (const local of this.locals) {
            const bucket = byAlbum.get(local.albumId);
            if (bucket) bucket.push(local);
            else byAlbum.set(local.albumId, [local]);
        }

        const byTarget = new Map<string, QueuedUpload[]>();
        for (const task of this.tasks) {
            const bucket = byTarget.get(task.destUri);
            if (bucket) bucket.push(task);
            else byTarget.set(task.destUri, [task]);
        }

        return [...this.albums]
            .sort((left, right) => left.path.localeCompare(right.path))
            .map((album) => {
                const groupTasks = byTarget.get(album.uri) ?? [];
                const mappings = (byAlbum.get(album.id) ?? []).map((local): Mapping => {
                    const own = groupTasks.filter((task) => task.localAlbumId === local.id);
                    const scan = scans.get(local.id) ?? null;
                    const tone: Tone = own.length > 0 ? 'upload' : live(scan ?? undefined) ? 'scan' : 'idle';
                    return {local, tone, job: scan, tasks: own};
                });
                const tone: Tone =
                    groupTasks.length > 0
                        ? 'upload'
                        : mappings.some((mapping) => mapping.tone === 'scan') || live(listings.get(album.id))
                          ? 'scan'
                          : 'idle';
                return {album, tone, job: listings.get(album.id) ?? null, mappings};
            });
    });

    /// List one album again. Its card shows the pass while it runs.
    async index(albumId: string): Promise<void> {
        await api.indexAlbum({albumId});
    }

    /// Scan one device folder again. Its card shows the pass while it runs.
    async scan(id: string): Promise<void> {
        await api.syncAlbum({id});
    }

    /// Stop one job by the key the card holds.
    async cancel(kind: JobKind, id: string): Promise<void> {
        await api.jobCancel({kind, id});
    }

    /// Point a folder at an album. The first scan starts with the mapping.
    async attach(uri: string, name: string, albumId: string): Promise<void> {
        await api.addLocalAlbum({uri, name, albumId});
    }

    /// Drop one mapping. Uploaded files stay in the album, and its queued rows
    /// go with the row that named it.
    async unbind(id: string): Promise<void> {
        await api.removeLocalAlbum({id});
    }
}

/// Wire a board to its page: the three views it draws from.
///
/// The effects live here, not on the class, because a rune only registers work
/// while a component is being set up, and this function is called exactly then.
export function startSyncBoard(board: SyncBoard): void {
    const grid = albumsView.use();
    const sources = localsView.use();
    const moving = runningView.use();

    // All three are subscriptions: they arrive whole once and are patched by
    // every commit, so a row added or removed on another screen is already in
    // these rows when this page comes back.
    $effect(() => {
        board.albums = grid.rows;
    });

    $effect(() => {
        board.locals = sources.rows;
    });

    $effect(() => {
        board.tasks = moving.rows;
    });
}
