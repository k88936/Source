/**
 * A client for a subscription connection, in SpacetimeDB's shape: declare a
 * query, get its rows once, then only the rows each commit changed.
 *
 * The protocol types are generated from the Rust crate that defines them, so the
 * two sides cannot drift. Nothing here knows about Tauri, a socket, or the
 * application: a [`Wire`] is a pair of halves and any transport can supply them.
 */
export {Client} from './client';
export type {Listeners, QueryDescriptor, Subscription} from './client';
export {useQuery} from './useQuery.svelte';
export type {DescribeError} from './useQuery.svelte';
export type {Wire} from './wire';
export type {ClientMessage, QueryRows, QuerySetUpdate, QueryUpdate, ServerMessage} from './protocol';
