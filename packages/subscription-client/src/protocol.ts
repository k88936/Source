/**
 * The wire, generated from the Rust crate that defines it.
 *
 * Every type here is a re-export: the messages are declared once, in Rust, so the
 * manager and this client cannot disagree about their shape.
 */
export type {ClientMessage} from './bindings/ClientMessage';
export type {QueryRows} from './bindings/QueryRows';
export type {QuerySetUpdate} from './bindings/QuerySetUpdate';
export type {QueryUpdate} from './bindings/QueryUpdate';
export type {ServerMessage} from './bindings/ServerMessage';
