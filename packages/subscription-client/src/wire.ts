import type {ClientMessage, ServerMessage} from './protocol';

/**
 * Where a connection's messages come and go.
 *
 * A wire is one pair of halves: send a client message, hand server messages to
 * a handler. Tauri commands, a websocket and an in-process channel are all the
 * same wire, so the client below never learns which one it has.
 */
export interface Wire<Q, R> {
    /** Start the connection, if the transport has to be started at all. */
    connect(): Promise<void>;
    send(message: ClientMessage<Q>): Promise<void>;
    /** Hand every server message to `handler`; resolves with the unsubscribe. */
    listen(handler: (message: ServerMessage<Q, R>) => void): Promise<() => void>;
}
