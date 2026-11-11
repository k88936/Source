import {changes} from '#lib/api/changes';
import type {AlbumMetadata} from '#lib/api/bindings/AlbumMetadata';
import type {Job} from '#lib/api/bindings/Job';
import type {JobKind} from '#lib/api/bindings/JobKind';
import type {LocalAlbumMetadata} from '#lib/api/bindings/LocalAlbumMetadata';
import type {QueuedUpload} from '#lib/api/bindings/QueuedUpload';
import {api} from '#lib/api/client';
import {app} from '#lib/app/app.svelte';

/// How a mapping reads at a glance. The line is the only place this shows:
/// bytes moving or not. A failure belongs to a file, never to a folder.
export type Tone = 'idle' | 'upload';

/// What one queue or one folder is doing, in the units the cards show.
export interface Tally {
    uploading: number;
    waiting: number;
    failed: number;
    speed: number;
    doneBytes: number;
    totalBytes: number;
    percent: number;
}

export interface Mapping {
    local: LocalAlbumMetadata;
    tone: Tone;
    /// Nothing left to send, and no scan looking for more.
    finished: boolean;
    tasks: QueuedUpload[];
    tally: Tally;
}

export interface Group {
    album: AlbumMetadata;
    mappings: Mapping[];
    tasks: QueuedUpload[];
    tally: Tally;
    eta: number | null;
}

/// The queue of the whole library, which the overview card owns.
export interface Queue extends Tally {
    remaining: number;
    eta: number | null;
    active: boolean;
}

function tally(list: QueuedUpload[]): Tally {
    let uploading = 0;
    let waiting = 0;
    let failed = 0;
    let speed = 0;
    let doneBytes = 0;
    let totalBytes = 0;
    for (const task of list) {
        if (task.state === 'failed') failed++;
        else if (task.state === 'queued') waiting++;
        else if (task.state !== 'done' && task.state !== 'cancelled') uploading++;
        speed += task.speed;
        doneBytes += task.uploaded;
        totalBytes += task.total;
    }
    return {
        uploading,
        waiting,
        failed,
        speed,
        doneBytes,
        totalBytes,
        percent: totalBytes > 0 ? doneBytes / totalBytes : 0,
    };
}

/// Only bytes on the wire animate, so movement always means this folder is
/// transferring. Anything else reads as linked and still, whatever the rows
/// say: a scan is not a state of its own.
function toneOf(tally: Tally): Tone {
    return tally.speed > 0 ? 'upload' : 'idle';
}

/// A folder is finished when it has nothing left to send and nothing left to
/// look for. One failed file does not finish a folder.
function finished(tally: Tally, scanning: boolean): boolean {
    return !scanning && tally.uploading === 0 && tally.waiting === 0 && tally.failed === 0;
}

const NOTHING: Tally = {
    uploading: 0,
    waiting: 0,
    failed: 0,
    speed: 0,
    doneBytes: 0,
    totalBytes: 0,
    percent: 0,
};

/**
 * The graph's data: what is mapped where, what each row is moving, and the
 * actions that change a mapping.
 *
 * The page and the modal both read one instance, so a folder opened in a modal
 * is answered by the same rows the card is drawn from.
 */
export class SyncBoard {
    albums = $state<AlbumMetadata[]>([]);
    locals = $state<LocalAlbumMetadata[]>([]);
    tasks = $state<QueuedUpload[]>([]);

    /// One cloud album drives a row: it is the destination, the local folders
    /// are the sources stacked beside it, and an album nobody points at still
    /// gets its row because it is still part of the library.
    ///
    /// Every queue row carries the folder that asked for the upload, so a
    /// folder is answered from its own rows and an album from their sum.
    groups: Group[] = $derived.by(() => {
        const scans = new Map<string, Job<JobKind>>();
        for (const job of app.jobs) if (job.kind === 'sync') scans.set(job.id, job);

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
                const groupTally = tally(groupTasks);
                const mappings = (byAlbum.get(album.id) ?? []).map((local): Mapping => {
                    const own = groupTasks.filter((task) => task.localAlbumId === local.id);
                    const ownTally = tally(own);
                    const scanning = scans.get(local.id)?.state === 'running';
                    return {
                        local,
                        tone: toneOf(ownTally),
                        finished: finished(ownTally, scanning),
                        tasks: own,
                        tally: ownTally,
                    };
                });
                return {
                    album,
                    mappings,
                    tasks: groupTasks,
                    tally: groupTally,
                    eta:
                        groupTally.speed > 0
                            ? Math.ceil(Math.max(0, groupTally.totalBytes - groupTally.doneBytes) / groupTally.speed)
                            : null,
                };
            });
    });

    /// A local row whose album is gone has nothing to point at, so it is drawn
    /// alone instead of vanishing with the line.
    orphans: Mapping[] = $derived(
        this.locals
            .filter((local) => !this.albums.some((album) => album.id === local.albumId))
            .map((local) => ({local, tone: 'idle', finished: false, tasks: [], tally: NOTHING})),
    );

    queue: Queue = $derived.by(() => {
        const total = tally(this.tasks);
        const remaining = Math.max(0, total.totalBytes - total.doneBytes);
        return {
            ...total,
            remaining,
            eta: total.speed > 0 ? Math.ceil(remaining / total.speed) : null,
            active: total.uploading + total.waiting + total.failed > 0,
        };
    });

    async loadAlbums(): Promise<void> {
        this.albums = await api.albumList();
    }

    async loadLocals(): Promise<void> {
        this.locals = await api.localAlbumList();
    }

    async loadTasks(): Promise<void> {
        this.tasks = await api.uploadTasks();
    }

    /// A mapping is a catalog change, so it comes and goes with the catalog.
    async loadCatalog(): Promise<void> {
        await Promise.all([this.loadAlbums(), this.loadLocals()]);
    }

    async reload(): Promise<void> {
        await Promise.all([this.loadCatalog(), this.loadTasks()]);
    }

    /// Point a folder at an album. The new row is enough to redraw the graph.
    async attach(uri: string, name: string, albumId: string): Promise<void> {
        await api.addLocalAlbum({uri, name, albumId});
        await this.loadLocals();
    }

    /// Drop one mapping. Uploaded files stay in the album.
    async unbind(id: string): Promise<void> {
        await api.removeLocalAlbum({id});
        await this.loadLocals();
    }
}

/// Wire a board to its page: the first read, live catalog changes, and the tick
/// that keeps the byte counters moving while anything is queued.
///
/// The effects live here, not on the class, because a rune only registers work
/// while a component is being set up, and this function is called exactly then.
export function startSyncBoard(board: SyncBoard): void {
    $effect(() => {
        void board.reload();
    });

    $effect(() => changes.subscribe(() => void board.loadCatalog()));

    $effect(() => {
        if (board.queue.uploading + board.queue.waiting === 0) return;
        const timer = setInterval(() => void board.loadTasks(), 1000);
        return () => clearInterval(timer);
    });
}
