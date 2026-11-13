import {onMount} from 'svelte';
import type {Client, QueryDescriptor, Subscription} from './client';

/** How a failure is turned into the line a page shows. */
export type DescribeError = (cause: unknown) => string;

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

    const show = () => {
        if (!live || !held) return;
        rows = held.rows();
        loading = false;
    };

    onMount(() => {
        client
            .subscribe(descriptor, {
                changed: show,
                failed: (message) => {
                    if (!live) return;
                    error = message;
                    rows = [];
                    loading = false;
                },
            })
            .then((subscription) => {
                held = subscription;
                show();
            })
            .catch((cause: unknown) => {
                if (!live) return;
                error = describeError(cause);
                loading = false;
            });
        return () => {
            live = false;
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
