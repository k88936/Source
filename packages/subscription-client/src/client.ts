import type {QueryRows, ServerMessage} from './protocol';
import type {Wire} from './wire';

/**
 * A query as a page declares it: its parameters and the order its rows come in.
 */
export interface QueryDescriptor<Q, R> {
    readonly query: Q;
    /**
     * The order rows are handed out in.
     *
     * Two surfaces showing one view must show it in the same order, so the order
     * belongs to the declaration and not to whichever component renders it.
     */
    readonly compare?: (a: R, b: R) => number;
}

/** What a page is told about the query it holds. */
export interface Listeners {
    /** The rows changed. */
    changed(): void;
    /** The catalog could not answer this query. */
    failed(error: string): void;
}

/** One live query, as a page holds it. */
export interface Subscription<R> {
    /** The rows now, in the descriptor's order. */
    rows(): R[];
    /** Ask for the whole set again, after the catalog failed to answer. */
    retry(): void;
    /** Stop: the rows stop arriving and the page's copy is dropped. */
    stop(): void;
}

/**
 * One open query set: its rows, and who to tell when they move.
 *
 * A page opens one set per query, so a failure or a teardown stays local to that
 * query. The protocol allows several queries in one set; the cache is one map
 * keyed by row id, so widening this later changes nothing above it.
 */
interface Open<R> {
    readonly cache: Map<string, R>;
    readonly compare?: (a: R, b: R) => number;
    readonly listeners: Listeners;
    /** Run once the snapshot is applied, or with the message that refused it. */
    readonly settle: (error?: string) => void;
}

/**
 * The connection and the copies it keeps.
 *
 * A subscriber gets one full snapshot from the catalog and then only the rows
 * each commit changed; this class is where those deltas are applied to a local
 * copy, so reading a page's rows is a memory read and never a round trip.
 */
export class Client<Q, R extends {id: string}> {
    readonly #wire: Wire<Q, R>;
    #connection: Promise<void> | undefined;
    #nextId = 1;
    readonly #open = new Map<number, Open<R>>();

    constructor(wire: Wire<Q, R>) {
        this.#wire = wire;
    }

    /**
     * Subscribe one query and keep it live.
     *
     * Resolves once the snapshot has been applied, so a page that awaits this
     * already has its rows; a refused subscription rejects instead.
     */
    async subscribe(descriptor: QueryDescriptor<Q, R>, listeners: Listeners): Promise<Subscription<R>> {
        const id = this.#nextId++;
        let settle: (error?: string) => void = () => {};
        const opened = new Promise<void>((resolve, reject) => {
            settle = (error) => (error === undefined ? resolve() : reject(new Error(error)));
        });
        this.#open.set(id, {cache: new Map(), compare: descriptor.compare, listeners, settle});
        try {
            await this.#connected();
            await this.#wire.send({type: 'subscribe', requestId: id, querySetId: id, queries: [descriptor.query]});
            await opened;
        } catch (cause) {
            this.#open.delete(id);
            throw cause;
        }
        return {
            rows: () => this.#rows(id),
            retry: () =>
                void this.#wire.send({type: 'subscribe', requestId: id, querySetId: id, queries: [descriptor.query]}),
            stop: () => this.#close(id),
        };
    }

    #connected(): Promise<void> {
        this.#connection ??= this.#listen();
        return this.#connection;
    }

    /** Listen before connecting: a snapshot must not arrive before the handler. */
    async #listen(): Promise<void> {
        await this.#wire.listen((message) => this.#receive(message));
        await this.#wire.connect();
    }

    #close(id: number): void {
        if (!this.#open.delete(id)) return;
        void this.#wire.send({type: 'unsubscribe', requestId: id, querySetId: id});
    }

    #rows(id: number): R[] {
        const open = this.#open.get(id);
        if (!open) return [];
        const rows = [...open.cache.values()];
        // TODO here is n log n but most elements are ordered, consider use insert sort if performance issue
        return open.compare ? rows.sort(open.compare) : rows;
    }

    #receive(message: ServerMessage<Q, R>): void {
        switch (message.type) {
            case 'subscribeApplied': {
                const open = this.#open.get(message.querySetId);
                if (!open) return;
                open.cache.clear();
                for (const rows of message.queries) this.#fill(open, rows);
                open.settle();
                open.listeners.changed();
                return;
            }
            case 'transactionUpdate': {
                for (const set of message.querySets) {
                    const open = this.#open.get(set.querySetId);
                    if (!open) continue;
                    // A page opens one query per set, so every update in the set
                    // belongs to it; the set id is the routing, not the query.
                    for (const update of set.queries) {
                        for (const row of update.inserts) open.cache.set(row.id, row);
                        for (const id of update.deletes) open.cache.delete(id);
                    }
                    open.listeners.changed();
                }
                return;
            }
            case 'unsubscribeApplied': {
                const open = this.#open.get(message.querySetId);
                if (!open) return;
                open.cache.clear();
                open.listeners.changed();
                return;
            }
            case 'subscriptionError': {
                const open = this.#open.get(message.querySetId);
                if (!open) return;
                open.settle(message.error);
                open.listeners.failed(message.error);
                return;
            }
        }
    }

    #fill(open: Open<R>, rows: QueryRows<Q, R>): void {
        for (const row of rows.rows) open.cache.set(row.id, row);
    }
}
