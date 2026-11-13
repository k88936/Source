import type {ClientMessage, ServerMessage, Wire} from 'subscription-client';
import {transport} from '#lib/api/transport';

/**
 * Which view a connection is for.
 *
 * The same word Rust names the manager with. One connection serves one view, so
 * a page has to say which one it is opening and which one a message belongs to.
 */
export type ViewName = 'albums' | 'assets';

/**
 * One view's connection to the catalog, over the Tauri command and event pair.
 *
 * A page never writes a statement: it names one of the views Rust declared,
 * receives every row once, and then only the rows each commit changed. The view
 * is also the event name, because each view has its own manager and its own
 * connection: a snapshot of the album grid cannot delay a timeline window's
 * deltas.
 */
export function wireFor<Q, R>(view: ViewName): Wire<Q, R> {
    return {
        connect: () => transport.invoke<void>('subscription_connect', {view}),
        send: (message: ClientMessage<Q>) =>
            transport.invoke<void>('subscription_send', {
                message: {view, ...(message as unknown as Record<string, unknown>)},
            }),
        listen: (handler: (message: ServerMessage<Q, R>) => void) =>
            transport.listen<ServerMessage<Q, R>>(`subscription://${view}`, handler),
    };
}
