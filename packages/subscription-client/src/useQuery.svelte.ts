import {onMount} from 'svelte';
import debounce from 'lodash/debounce';
import type {Client, QueryDescriptor, Subscription} from './client';

/** How a failure is turned into the line a page shows. */
export type DescribeError = (cause: unknown) => string;

/**
 * How long commits are folded together before the rows are read again.
 *
 * A commit arrives per write, and a page being filled writes one row at a time,
 * so the card list would otherwise be rebuilt once per row. Waiting this long
 * paints the same picture about eight times a second; a stream that never
 * pauses still moves because the wait is capped.
 */
const SETTLE_MS = 120;
const SETTLE_MAX_MS = 240;

/**
 * One query, live, for as long as the component is mounted.
 *
 * Mounting subscribes and unmounting drops the set; every read afterwards is
 * from the client's local copy, so a page renders a commit without asking the
 * catalog for anything.
 */
export function useQuery<Q, R extends {id: string}>(
    client: Client<Q, R>,
    descriptor: QueryDescriptor<Q, R>,
    describeError: DescribeError = (cause) => String(cause),
) {
    let rows = $state<R[]>([]);
    let loading = $state(true);
    let error = $state('');
    let held: Subscription<R> | undefined;
    let live = true;

    const read = () => {
        if (!live || !held) return;
        rows = held.rows();
        loading = false;
    };
    const show = debounce(read, SETTLE_MS, {maxWait: SETTLE_MAX_MS});

    onMount(() => {
        client
            .subscribe(descriptor, {
                changed: show,
                failed: (message) => {
                    if (!live) return;
                    show.cancel();
                    error = message;
                    rows = [];
                    loading = false;
                },
            })
            .then((subscription) => {
                held = subscription;
                read();
            })
            .catch((cause: unknown) => {
                if (!live) return;
                error = describeError(cause);
                loading = false;
            });
        return () => {
            live = false;
            show.cancel();
            held?.stop();
        };
    });

    return {
        get rows(): R[] {
            return rows;
        },
        get loading(): boolean {
            return loading;
        },
        get error(): string {
            return error;
        },
        /** Ask again, after the catalog could not answer this query. */
        retry(): void {
            if (!held) return;
            error = '';
            loading = true;
            held.retry();
        },
    };
}
